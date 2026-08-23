//! This module owns ordered version-2 migration namespace admission.

use std::io;

use cap_std::fs::Dir;

use super::filesystem_migration_namespace_directory::{
    PinnedMigrationDirectory, ambiguous, exact_membership, optional_directory,
    require_allowed_membership, require_directory, require_empty, require_exact_membership,
    require_regular, required_directory,
};
use super::filesystem_migration_reader_fence;

const WRITER_LOCK: &str = "writer.lock";
const STAGING: &str = "staging";
const SEGMENTS: &str = "segments";
const CATALOGS: &str = "catalogs";
const HEAD: &str = "HEAD";
const INTENT: &str = "migration.intent";
const READER_LOCK: &str = "reader.lock";
const RETENTION: &str = "retention";
const ROOTS: &str = "roots";
const MANIFESTS: &str = "manifests";
const GC: &str = "gc";
const RECOVERY: &str = "recovery";
const DISPOSITIONS: &str = "dispositions";

const BEFORE_READER: [&str; 6] = [WRITER_LOCK, STAGING, SEGMENTS, CATALOGS, HEAD, INTENT];
const AFTER_READER: [&str; 7] = [
    WRITER_LOCK,
    STAGING,
    SEGMENTS,
    CATALOGS,
    HEAD,
    INTENT,
    READER_LOCK,
];
const PREFIX_ROOT: [&str; 10] = [
    WRITER_LOCK,
    STAGING,
    SEGMENTS,
    CATALOGS,
    HEAD,
    INTENT,
    READER_LOCK,
    RETENTION,
    GC,
    RECOVERY,
];
const MARKER_ROOT: [&str; 11] = [
    WRITER_LOCK,
    STAGING,
    SEGMENTS,
    CATALOGS,
    HEAD,
    INTENT,
    READER_LOCK,
    RETENTION,
    GC,
    RECOVERY,
    "FORMAT",
];
const RECEIPT_ROOT: [&str; 12] = [
    WRITER_LOCK,
    STAGING,
    SEGMENTS,
    CATALOGS,
    HEAD,
    INTENT,
    READER_LOCK,
    RETENTION,
    GC,
    RECOVERY,
    "FORMAT",
    "migration.receipt",
];

pub(super) fn admit_reader_fence(root: &Dir) -> io::Result<()> {
    let before = exact_membership(root, &BEFORE_READER)?;
    let after = exact_membership(root, &AFTER_READER)?;
    if !before && !after {
        return Err(ambiguous("reader-fence predecessor namespace disagreed"));
    }
    let file = if before {
        filesystem_migration_reader_fence::create(root)?
    } else {
        filesystem_migration_reader_fence::open(root)?
    };
    filesystem_migration_reader_fence::verify(root, &file)?;
    file.sync_all()?;
    filesystem_migration_reader_fence::verify(root, &file)?;
    verify_reader_root(root)
}

pub(super) fn admit_namespace_prefix(root: &Dir) -> io::Result<()> {
    preflight_prefix(root)?;
    let retention = PinnedMigrationDirectory::admit(root, RETENTION)?;
    let roots = PinnedMigrationDirectory::admit(retention.directory(), ROOTS)?;
    let manifests = PinnedMigrationDirectory::admit(retention.directory(), MANIFESTS)?;
    let gc = PinnedMigrationDirectory::admit(root, GC)?;
    let recovery = PinnedMigrationDirectory::admit(root, RECOVERY)?;
    let dispositions = PinnedMigrationDirectory::admit(recovery.directory(), DISPOSITIONS)?;
    roots.verify(retention.directory())?;
    manifests.verify(retention.directory())?;
    dispositions.verify(recovery.directory())?;
    retention.verify(root)?;
    gc.verify(root)?;
    recovery.verify(root)?;
    verify_namespace_prefix(root)
}

pub(super) fn verify_intent_root(root: &Dir) -> io::Result<()> {
    require_v1_and_intent(root)?;
    require_exact_membership(root, &BEFORE_READER)
}

pub(super) fn verify_reader_root(root: &Dir) -> io::Result<()> {
    require_v1_and_intent(root)?;
    let reader = filesystem_migration_reader_fence::open(root)?;
    filesystem_migration_reader_fence::verify(root, &reader)?;
    require_exact_membership(root, &AFTER_READER)
}

pub(super) fn verify_namespace_prefix(root: &Dir) -> io::Result<()> {
    verify_prefix_directories(root)?;
    require_exact_membership(root, &PREFIX_ROOT)
}

pub(super) fn verify_namespace_contents(root: &Dir) -> io::Result<()> {
    verify_prefix_directories(root)
}

pub(super) fn verify_marker_view(root: &Dir) -> io::Result<()> {
    verify_marker_contents(root)?;
    require_exact_membership(root, &MARKER_ROOT)
}

pub(super) fn verify_marker_contents(root: &Dir) -> io::Result<()> {
    verify_prefix_directories(root)?;
    require_regular(root, "FORMAT", Some(96))
}

pub(super) fn verify_receipt_view(root: &Dir) -> io::Result<()> {
    verify_prefix_directories(root)?;
    require_regular(root, "FORMAT", Some(96))?;
    require_regular(root, "migration.receipt", Some(256))?;
    require_exact_membership(root, &RECEIPT_ROOT)
}

fn preflight_prefix(root: &Dir) -> io::Result<()> {
    require_allowed_membership(root, &PREFIX_ROOT)?;
    require_base_namespace(root)?;
    let retention = optional_directory(root, RETENTION)?;
    let gc = optional_directory(root, GC)?;
    let recovery = optional_directory(root, RECOVERY)?;
    let retention_complete = preflight_retention(retention.as_ref())?;
    if gc.is_some() && !retention_complete {
        return Err(ambiguous("gc appeared before the retention prefix"));
    }
    if recovery.is_some() && gc.is_none() {
        return Err(ambiguous("recovery appeared before the gc prefix"));
    }
    if let Some(directory) = gc.as_ref() {
        require_empty(directory.directory())?;
    }
    preflight_recovery(recovery.as_ref())
}

fn require_base_namespace(root: &Dir) -> io::Result<()> {
    require_v1_and_intent(root)?;
    let reader = filesystem_migration_reader_fence::open(root)?;
    filesystem_migration_reader_fence::verify(root, &reader)
}

fn require_v1_and_intent(root: &Dir) -> io::Result<()> {
    require_regular(root, WRITER_LOCK, None)?;
    require_directory(root, STAGING)?;
    require_directory(root, SEGMENTS)?;
    require_directory(root, CATALOGS)?;
    require_regular(root, HEAD, Some(128))?;
    require_regular(root, INTENT, Some(256))
}

fn verify_prefix_directories(root: &Dir) -> io::Result<()> {
    require_base_namespace(root)?;
    let retention = required_directory(root, RETENTION)?;
    let roots = required_directory(retention.directory(), ROOTS)?;
    let manifests = required_directory(retention.directory(), MANIFESTS)?;
    let gc = required_directory(root, GC)?;
    let recovery = required_directory(root, RECOVERY)?;
    let dispositions = required_directory(recovery.directory(), DISPOSITIONS)?;
    require_empty(roots.directory())?;
    require_empty(manifests.directory())?;
    require_empty(gc.directory())?;
    require_empty(dispositions.directory())?;
    require_exact_membership(retention.directory(), &[ROOTS, MANIFESTS])?;
    require_exact_membership(recovery.directory(), &[DISPOSITIONS])
}

fn preflight_retention(retention: Option<&PinnedMigrationDirectory>) -> io::Result<bool> {
    let Some(retention) = retention else {
        return Ok(false);
    };
    require_allowed_membership(retention.directory(), &[ROOTS, MANIFESTS])?;
    let roots = optional_directory(retention.directory(), ROOTS)?;
    let manifests = optional_directory(retention.directory(), MANIFESTS)?;
    if manifests.is_some() && roots.is_none() {
        return Err(ambiguous("retention manifests appeared before roots"));
    }
    if let Some(directory) = roots.as_ref() {
        require_empty(directory.directory())?;
    }
    if let Some(directory) = manifests.as_ref() {
        require_empty(directory.directory())?;
    }
    Ok(roots.is_some() && manifests.is_some())
}

fn preflight_recovery(recovery: Option<&PinnedMigrationDirectory>) -> io::Result<()> {
    let Some(recovery) = recovery else {
        return Ok(());
    };
    require_allowed_membership(recovery.directory(), &[DISPOSITIONS])?;
    if let Some(dispositions) = optional_directory(recovery.directory(), DISPOSITIONS)? {
        require_empty(dispositions.directory())?;
    }
    Ok(())
}
