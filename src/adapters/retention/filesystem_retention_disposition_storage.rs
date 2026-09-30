//! This module owns the filesystem effect of every disposition phase.

use std::io;

use cap_fs_ext::DirExt;
use cap_std::fs::Dir;

use super::filesystem_retention_authority::FilesystemRetentionPublicationAuthority;
use super::filesystem_retention_current::read_exact_optional;
use super::filesystem_retention_disposition::PoolEntry;
use super::filesystem_retention_pool_name as pool_name;
use super::filesystem_retention_stage::{FilesystemRetentionStage, invalid_data};
use super::{RecoveryDispositionStorage, RecoveryDispositionTarget, RetentionRecoveryStorage};
use crate::adapters::filesystem_catalog_artifact::synchronize_directory;
use crate::adapters::filesystem_exact_record as exact_record;

fn no_disposition() -> io::Error {
    invalid_data("no disposition is in progress")
}

/// Removes `name` from `directory` after proving it still holds `expected`.
fn unlink_verified(directory: &Dir, name: &str, expected: &[u8]) -> io::Result<()> {
    let observed = read_exact_optional(directory, name, expected.len())?
        .ok_or_else(|| invalid_data("retired pool entry is already absent"))?;
    if observed.as_ref() != expected {
        return Err(invalid_data(
            "retired pool entry bytes disagree with the receipt",
        ));
    }
    directory.remove_file(name)?;
    exact_record::require_absent(directory, name)
        .map_err(|_source| invalid_data("retired pool entry remained visible"))
}

impl RecoveryDispositionStorage for FilesystemRetentionPublicationAuthority {
    fn write_disposition_stage(&mut self) -> io::Result<()> {
        let context = self.disposition.as_mut().ok_or_else(no_disposition)?;
        context.stage = Some(FilesystemRetentionStage::create(
            &context.recovery,
            pool_name::DISPOSITION_STAGE,
            context.receipt.encoded(),
        )?);
        Ok(())
    }

    fn synchronize_disposition_stage(&mut self) -> io::Result<()> {
        let context = self.disposition.as_mut().ok_or_else(no_disposition)?;
        if context.stage.is_none() {
            context.stage = Some(FilesystemRetentionStage::reopen(
                &context.recovery,
                pool_name::DISPOSITION_STAGE,
                context.receipt.encoded(),
            )?);
        }
        let stage = context.stage.as_ref().ok_or_else(no_disposition)?;
        stage.synchronize(&context.recovery)
    }

    fn link_disposition_receipt(&mut self) -> io::Result<()> {
        let context = self.disposition.as_ref().ok_or_else(no_disposition)?;
        let stage = context.stage.as_ref().ok_or_else(no_disposition)?;
        stage.link(&context.recovery, &context.dispositions, &context.name)
    }

    fn synchronize_dispositions(&mut self) -> io::Result<()> {
        let context = self.disposition.as_ref().ok_or_else(no_disposition)?;
        synchronize_directory(&context.dispositions)
    }

    fn remove_disposition_stage(&mut self) -> io::Result<()> {
        let context = self.disposition.as_mut().ok_or_else(no_disposition)?;
        if context.stage.is_none() {
            context.stage = Some(FilesystemRetentionStage::reopen(
                &context.recovery,
                pool_name::DISPOSITION_STAGE,
                context.receipt.encoded(),
            )?);
        }
        let stage = context.stage.take().ok_or_else(no_disposition)?;
        stage.remove(&context.recovery, &context.dispositions, &context.name)
    }

    fn synchronize_recovery(&mut self) -> io::Result<()> {
        let context = self.disposition.as_ref().ok_or_else(no_disposition)?;
        synchronize_directory(&context.recovery)
    }

    fn remove_retained_stage(&mut self) -> io::Result<()> {
        let target = self.disposition.as_ref().ok_or_else(no_disposition)?.target;
        match target {
            RecoveryDispositionTarget::Root => RetentionRecoveryStorage::remove_root_stage(self),
            RecoveryDispositionTarget::Manifest => {
                RetentionRecoveryStorage::remove_manifest_stage(self)
            }
        }
    }

    fn synchronize_retention_after_disposition(&mut self) -> io::Result<()> {
        synchronize_directory(&self.retention)
    }

    fn remove_pool_entry(&mut self) -> io::Result<()> {
        let context = self.disposition.as_ref().ok_or_else(no_disposition)?;
        match &context.pool {
            PoolEntry::Root { namespace, name } => {
                let directory = self.roots.open_dir_nofollow(namespace)?;
                unlink_verified(&directory, name, &context.artifact)?;
                synchronize_directory(&directory)?;
                if directory.entries()?.next().is_none() {
                    drop(directory);
                    self.roots.remove_dir(namespace)?;
                }
                Ok(())
            }
            PoolEntry::Manifest { name } => {
                unlink_verified(&self.manifests, name, &context.artifact)
            }
        }
    }

    fn synchronize_pool(&mut self) -> io::Result<()> {
        let context = self.disposition.as_ref().ok_or_else(no_disposition)?;
        match context.pool {
            PoolEntry::Root { .. } => synchronize_directory(&self.roots),
            PoolEntry::Manifest { .. } => synchronize_directory(&self.manifests),
        }
    }
}
