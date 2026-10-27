//! This module owns bounded admission of the initialization namespace.

use std::ffi::OsStr;
use std::io;

use super::filesystem_namespace_refusal::{
    self as namespace, FilesystemEntryKind, FilesystemNamespaceRefusal,
};
use cap_fs_ext::DirExt;
use cap_std::fs::Dir;

const LOCK_NAME: &str = "writer.lock";
const STAGING_NAME: &str = "staging";
const SEGMENTS_NAME: &str = "segments";
const CATALOGS_NAME: &str = "catalogs";
const HEAD_NAME: &str = "HEAD";
const READER_LOCK_NAME: &str = "reader.lock";
const MARKER_NAME: &str = "FORMAT";
const MARKER_STAGE_NAME: &str = "FORMAT.next";
const RETENTION_NAME: &str = "retention";
const GC_NAME: &str = "gc";
const RECOVERY_NAME: &str = "recovery";
const ROOTS_NAME: &str = "roots";
const MANIFESTS_NAME: &str = "manifests";
const DISPOSITIONS_NAME: &str = "dispositions";
const STORE_NAMES: [&str; 10] = [
    LOCK_NAME,
    STAGING_NAME,
    SEGMENTS_NAME,
    CATALOGS_NAME,
    HEAD_NAME,
    READER_LOCK_NAME,
    MARKER_NAME,
    RETENTION_NAME,
    GC_NAME,
    RECOVERY_NAME,
];
const NEXT_HEAD_NAME: &str = "head.next";
const STAGED_STORE_NAMES: [&str; 11] = [
    LOCK_NAME,
    STAGING_NAME,
    SEGMENTS_NAME,
    CATALOGS_NAME,
    HEAD_NAME,
    NEXT_HEAD_NAME,
    READER_LOCK_NAME,
    MARKER_NAME,
    RETENTION_NAME,
    GC_NAME,
    RECOVERY_NAME,
];

/// Admits the exact committed store root namespace.
///
/// `writer.lock`, `staging`, `segments`, `catalogs`, the persistent reader
/// fence, the format marker, and all three protocol directories must be
/// present; catalog `HEAD` is optional, because an initialized store holds
/// none until its first publication. A root that still holds `FORMAT.next`
/// refuses as `InitializationIncomplete` and a root without `FORMAT` as
/// `MissingFormatMarker`. Any other entry is unrecoverable ambiguity.
pub(super) fn admit_store(directory: &Dir) -> io::Result<()> {
    admit_store_required(directory)?;
    admit_membership(directory, &STORE_NAMES)?;
    admit_protocol_directories(directory)
}

/// Admits a committed store root that may retain a catalog `head.next`.
///
/// `head.next` is the catalog-protocol head stage. A reader selects only
/// `HEAD`, so a retained or in-flight `head.next` does not change its view, and
/// catalog-stage recovery must be able to open the store that holds one. Writer admission keeps the
/// exact law of [`admit_store`] and refuses it. Every other entry and
/// every protocol directory is admitted exactly as [`admit_store`] does.
pub(super) fn admit_store_with_catalog_stage(directory: &Dir) -> io::Result<()> {
    admit_store_required(directory)?;
    admit_optional_file(directory, NEXT_HEAD_NAME)?;
    admit_membership(directory, &STAGED_STORE_NAMES)?;
    admit_protocol_directories(directory)
}

fn admit_store_required(directory: &Dir) -> io::Result<()> {
    admit_committed(directory)?;
    admit_store_layout(directory)
}

/// Admits every committed root entry by kind; catalog `HEAD` is optional.
fn admit_store_layout(directory: &Dir) -> io::Result<()> {
    admit_required_file(directory, LOCK_NAME)?;
    admit_required_directory(directory, STAGING_NAME)?;
    admit_required_directory(directory, SEGMENTS_NAME)?;
    admit_required_directory(directory, CATALOGS_NAME)?;
    admit_optional_file(directory, HEAD_NAME)?;
    admit_required_file(directory, READER_LOCK_NAME)?;
    admit_required_file(directory, MARKER_NAME)?;
    admit_required_directory(directory, RETENTION_NAME)?;
    admit_required_directory(directory, GC_NAME)?;
    admit_required_directory(directory, RECOVERY_NAME)
}

/// Admits the nested protocol directories initialization created.
///
/// `retention` must carry both immutable pools (its head and stages belong to
/// retention publication); `gc` must be empty until `KEEP-GC-001` implements
/// its records; `recovery` must hold exactly an empty `dispositions`. A root
/// that drifted after initialization refuses here rather than as a later
/// pinning failure.
fn admit_protocol_directories(directory: &Dir) -> io::Result<()> {
    let retention = directory.open_dir_nofollow(RETENTION_NAME)?;
    admit_required_directory(&retention, ROOTS_NAME)?;
    admit_required_directory(&retention, MANIFESTS_NAME)?;
    let gc = directory.open_dir_nofollow(GC_NAME)?;
    admit_membership(&gc, &[])?;
    let recovery = directory.open_dir_nofollow(RECOVERY_NAME)?;
    admit_required_directory(&recovery, DISPOSITIONS_NAME)?;
    admit_membership(&recovery, &[DISPOSITIONS_NAME])?;
    let dispositions = recovery.open_dir_nofollow(DISPOSITIONS_NAME)?;
    admit_membership(&dispositions, &[])
}

fn admit_optional_file(directory: &Dir, name: &str) -> io::Result<()> {
    admit_optional_kind(directory, name, FilesystemEntryKind::File)
}

fn admit_required_file(directory: &Dir, name: &str) -> io::Result<()> {
    admit_required_kind(directory, name, FilesystemEntryKind::File)
}

fn admit_required_directory(directory: &Dir, name: &str) -> io::Result<()> {
    admit_required_kind(directory, name, FilesystemEntryKind::Directory)
}

fn admit_required_kind(
    directory: &Dir,
    name: &str,
    expected: FilesystemEntryKind,
) -> io::Result<()> {
    let metadata = directory.symlink_metadata(name)?;
    namespace::require_kind(metadata.file_type(), expected)
}

fn admit_optional_kind(
    directory: &Dir,
    name: &str,
    expected: FilesystemEntryKind,
) -> io::Result<()> {
    match directory.symlink_metadata(name) {
        Ok(metadata) => namespace::require_kind(metadata.file_type(), expected),
        Err(source) if source.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(source) => Err(source),
    }
}

fn admit_membership(directory: &Dir, canonical_names: &[&str]) -> io::Result<()> {
    let mut observed = 0_usize;
    for entry in directory.entries()? {
        let entry = entry?;
        observed = observed
            .checked_add(1)
            .ok_or_else(|| io::Error::other("namespace entry count overflow"))?;
        if observed > canonical_names.len() {
            return Err(ambiguous_namespace());
        }
        let name = entry.file_name();
        if !is_canonical(&name, canonical_names) {
            return Err(ambiguous_namespace());
        }
    }
    Ok(())
}

fn is_canonical(name: &OsStr, canonical_names: &[&str]) -> bool {
    canonical_names.iter().any(|candidate| name == *candidate)
}

fn ambiguous_namespace() -> io::Error {
    FilesystemNamespaceRefusal::UnexpectedEntry.into_io()
}

/// Admits a committed root whose `FORMAT.next` cleanup was interrupted.
///
/// Initialization uses this law only after it has proven that `FORMAT` and
/// `FORMAT.next` name one canonical marker inode. Every committed entry is
/// admitted as [`admit_store_with_catalog_stage`] admits it, and
/// `FORMAT.next` is additionally required.
pub(super) fn admit_store_marker_linked(directory: &Dir) -> io::Result<()> {
    admit_required_file(directory, MARKER_STAGE_NAME)?;
    admit_store_layout(directory)?;
    admit_optional_file(directory, NEXT_HEAD_NAME)?;
    let mut allowed: Vec<&str> = STAGED_STORE_NAMES.to_vec();
    allowed.push(MARKER_STAGE_NAME);
    admit_membership(directory, &allowed)?;
    admit_protocol_directories(directory)
}

/// Refuses an uncommitted root with a typed reason before any other law.
fn admit_committed(directory: &Dir) -> io::Result<()> {
    match directory.symlink_metadata(MARKER_STAGE_NAME) {
        Ok(_) => return Err(FilesystemNamespaceRefusal::InitializationIncomplete.into_io()),
        Err(source) if source.kind() == io::ErrorKind::NotFound => {}
        Err(source) => return Err(source),
    }
    match directory.symlink_metadata(MARKER_NAME) {
        Ok(_) => Ok(()),
        Err(source) if source.kind() == io::ErrorKind::NotFound => {
            Err(FilesystemNamespaceRefusal::MissingFormatMarker.into_io())
        }
        Err(source) => Err(source),
    }
}
