//! This module owns observing migration residue on one pinned root.

use std::io;

use cap_fs_ext::DirExt;
use cap_std::fs::Dir;

use super::FilesystemMigrationRecoveryRefusal as Refusal;
use super::filesystem_migration_recovery_refusal::{invalid, kind};
use super::{
    FORMAT_MARKER_LENGTH, MIGRATION_INTENT_LENGTH, MIGRATION_RECEIPT_LENGTH, StoreMigrationResidue,
};
use crate::adapters::filesystem_exact_record::{self as exact_record, ExactRecordError};

const READER_LOCK: &str = "reader.lock";

/// Observes every fixed migration name without mutation.
///
/// Records and stages are read without following links and bounded to one
/// byte more than their canonical length, so an overlong file stays
/// distinguishable from an exact one. A wrong kind refuses.
pub(super) fn observe(root: &Dir) -> io::Result<StoreMigrationResidue> {
    Ok(StoreMigrationResidue {
        intent_stage: bounded_file(root, "migration.intent.next", MIGRATION_INTENT_LENGTH)?,
        intent: bounded_file(root, "migration.intent", MIGRATION_INTENT_LENGTH)?,
        reader_fence: reader_fence(root)?,
        namespace_prefix: namespace_prefix(root)?,
        marker_stage: bounded_file(root, "FORMAT.next", FORMAT_MARKER_LENGTH)?,
        marker: bounded_file(root, "FORMAT", FORMAT_MARKER_LENGTH)?,
        receipt_stage: bounded_file(root, "migration.receipt.next", MIGRATION_RECEIPT_LENGTH)?,
        receipt: bounded_file(root, "migration.receipt", MIGRATION_RECEIPT_LENGTH)?,
    })
}

fn bounded_file(root: &Dir, name: &str, length: usize) -> io::Result<Option<Vec<u8>>> {
    let bound = length
        .checked_add(1)
        .ok_or_else(|| invalid(Refusal::ResidueBoundOverflow { length }))?;
    exact_record::read_bounded_optional(root, name, bound).map_err(ExactRecordError::into_io)
}

fn reader_fence(root: &Dir) -> io::Result<bool> {
    match root.symlink_metadata(READER_LOCK) {
        Err(source) if source.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(source) => Err(source),
        Ok(metadata) if metadata.is_file() && metadata.len() == 0 => Ok(true),
        Ok(metadata) => Err(invalid(Refusal::ReaderFence {
            observed_kind: kind(&metadata),
            observed_length: metadata.len(),
        })),
    }
}

fn namespace_prefix(root: &Dir) -> io::Result<[bool; 6]> {
    let retention = optional_directory(root, "retention")?;
    let (roots, manifests) = match retention.as_ref() {
        Some(retention) => (
            optional_directory(retention, "roots")?.is_some(),
            optional_directory(retention, "manifests")?.is_some(),
        ),
        None => (false, false),
    };
    let gc = optional_directory(root, "gc")?.is_some();
    let recovery = optional_directory(root, "recovery")?;
    let dispositions = match recovery.as_ref() {
        Some(recovery) => optional_directory(recovery, "dispositions")?.is_some(),
        None => false,
    };
    Ok([
        retention.is_some(),
        roots,
        manifests,
        gc,
        recovery.is_some(),
        dispositions,
    ])
}

fn optional_directory(parent: &Dir, name: &str) -> io::Result<Option<Dir>> {
    match parent.symlink_metadata(name) {
        Err(source) if source.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(source) => Err(source),
        Ok(metadata) if metadata.is_dir() => parent.open_dir_nofollow(name).map(Some),
        Ok(metadata) => Err(invalid(Refusal::NamespaceKind {
            observed_kind: kind(&metadata),
        })),
    }
}
