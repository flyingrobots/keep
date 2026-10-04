//! This module owns the filesystem effect of every disposition phase.

use std::io;

use cap_fs_ext::DirExt;
use cap_std::fs::Dir;

use super::filesystem_retention_authority::FilesystemRetentionPublicationAuthority;
use super::filesystem_retention_current::read_exact_optional;
use super::filesystem_retention_disposition::PoolEntry;
use super::filesystem_retention_pool_name as pool_name;
use super::filesystem_retention_stage::{FilesystemRetentionStage, invalid_data};
use super::{
    RecoveryDispositionTarget, RetentionRecoveryStorage, RetentionStorageBoundary as Boundary,
};
use crate::adapters::filesystem_catalog_artifact::synchronize_directory;
use crate::adapters::filesystem_exact_record as exact_record;

fn no_disposition() -> io::Error {
    invalid_data(super::FilesystemRetentionStageRefusal::NoDisposition)
}

/// Removes `name` from `directory` after proving it still holds `expected`.
fn unlink_verified(
    authority: &FilesystemRetentionPublicationAuthority,
    directory: &Dir,
    name: &str,
    expected: &[u8],
) -> io::Result<()> {
    authority.disposition_checkpoint(Boundary::PoolVerification)?;
    let observed = read_exact_optional(directory, name, expected.len())?
        .ok_or_else(|| invalid_data(super::FilesystemRetentionStageRefusal::PoolEntryAbsent))?;
    if observed.as_ref() != expected {
        return Err(invalid_data(
            super::FilesystemRetentionStageRefusal::PoolEntryChanged,
        ));
    }
    authority.disposition_checkpoint(Boundary::PoolUnlink)?;
    directory.remove_file(name)?;
    authority.disposition_checkpoint(Boundary::PoolAbsence)?;
    exact_record::require_absent(directory, name).map_err(exact_record::ExactRecordError::into_io)
}

impl FilesystemRetentionPublicationAuthority {
    pub(super) fn store_write_disposition_stage(&mut self) -> io::Result<()> {
        let context = self.disposition.as_mut().ok_or_else(no_disposition)?;
        context.stage = Some(FilesystemRetentionStage::create(
            &context.recovery,
            pool_name::DISPOSITION_STAGE,
            context.receipt.encoded(),
        )?);
        Ok(())
    }

    pub(super) fn store_synchronize_disposition_stage(&mut self) -> io::Result<()> {
        let context = self.disposition.as_mut().ok_or_else(no_disposition)?;
        if context.stage.is_none() {
            context.stage = Some(FilesystemRetentionStage::reopen(
                &context.recovery,
                pool_name::DISPOSITION_STAGE,
                context.receipt.encoded(),
                context
                    .stage_observation
                    .as_ref()
                    .ok_or_else(no_disposition)?
                    .identity(),
            )?);
        }
        let stage = context.stage.as_ref().ok_or_else(no_disposition)?;
        stage.synchronize(&context.recovery).map_err(Into::into)
    }

    pub(super) fn store_link_disposition_receipt(&mut self) -> io::Result<()> {
        let context = self.disposition.as_ref().ok_or_else(no_disposition)?;
        let stage = context.stage.as_ref().ok_or_else(no_disposition)?;
        stage
            .link(&context.recovery, &context.dispositions, &context.name)
            .map(|_outcome| ())
            .map_err(Into::into)
    }

    pub(super) fn store_synchronize_dispositions(&mut self) -> io::Result<()> {
        self.disposition_checkpoint(Boundary::DispositionSynchronization)?;
        let context = self.disposition.as_ref().ok_or_else(no_disposition)?;
        synchronize_directory(&context.dispositions)
    }

    pub(super) fn store_remove_disposition_stage(&mut self) -> io::Result<()> {
        let context = self.disposition.as_mut().ok_or_else(no_disposition)?;
        if context.stage.is_none() {
            context.stage = Some(FilesystemRetentionStage::reopen(
                &context.recovery,
                pool_name::DISPOSITION_STAGE,
                context.receipt.encoded(),
                context
                    .stage_observation
                    .as_ref()
                    .ok_or_else(no_disposition)?
                    .identity(),
            )?);
        }
        let stage = context.stage.take().ok_or_else(no_disposition)?;
        stage
            .remove(&context.recovery, &context.dispositions, &context.name)
            .map_err(Into::into)
    }

    pub(super) fn store_synchronize_recovery(&mut self) -> io::Result<()> {
        self.disposition_checkpoint(Boundary::RecoverySynchronization)?;
        let context = self.disposition.as_ref().ok_or_else(no_disposition)?;
        synchronize_directory(&context.recovery)
    }

    pub(super) fn store_remove_retained_stage(&mut self) -> io::Result<()> {
        let target = self.disposition.as_ref().ok_or_else(no_disposition)?.target;
        match target {
            RecoveryDispositionTarget::Root => {
                RetentionRecoveryStorage::remove_root_stage(self).map_err(Into::into)
            }
            RecoveryDispositionTarget::Manifest => {
                RetentionRecoveryStorage::remove_manifest_stage(self).map_err(Into::into)
            }
        }
    }

    pub(super) fn store_synchronize_retention_after_disposition(&mut self) -> io::Result<()> {
        self.disposition_checkpoint(Boundary::RetentionSynchronization)?;
        synchronize_directory(&self.retention)
    }

    pub(super) fn store_remove_pool_entry(&mut self) -> io::Result<()> {
        let context = self.disposition.as_ref().ok_or_else(no_disposition)?;
        match &context.pool {
            PoolEntry::Root { namespace, name } => {
                let directory = self.roots.open_dir_nofollow(namespace)?;
                unlink_verified(self, &directory, name, &context.artifact)?;
                self.disposition_checkpoint(Boundary::NamespaceSynchronization)?;
                synchronize_directory(&directory)?;
                self.disposition_checkpoint(Boundary::NamespaceEnumeration)?;
                if directory.entries()?.next().is_none() {
                    drop(directory);
                    self.disposition_checkpoint(Boundary::NamespaceRemoval)?;
                    self.roots.remove_dir(namespace)?;
                    self.disposition_checkpoint(Boundary::NamespaceAbsence)?;
                }
                Ok(())
            }
            PoolEntry::Manifest { name } => {
                unlink_verified(self, &self.manifests, name, &context.artifact)
            }
        }
    }

    pub(super) fn store_synchronize_pool(&mut self) -> io::Result<()> {
        self.disposition_checkpoint(Boundary::PoolSynchronization)?;
        let context = self.disposition.as_ref().ok_or_else(no_disposition)?;
        match context.pool {
            PoolEntry::Root { .. } => synchronize_directory(&self.roots),
            PoolEntry::Manifest { .. } => synchronize_directory(&self.manifests),
        }
    }
}
