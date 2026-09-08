//! This module owns bounded admission of the initialization namespace.

use std::ffi::OsStr;
use std::io;

use cap_fs_ext::DirExt;
use cap_std::fs::Dir;

const LOCK_NAME: &str = "writer.lock";
const STAGING_NAME: &str = "staging";
const SEGMENTS_NAME: &str = "segments";
const CATALOGS_NAME: &str = "catalogs";
const HEAD_NAME: &str = "HEAD";
const INITIALIZATION_NAMES: [&str; 4] = [LOCK_NAME, STAGING_NAME, SEGMENTS_NAME, CATALOGS_NAME];
const PUBLISHED_NAMES: [&str; 5] = [
    LOCK_NAME,
    STAGING_NAME,
    SEGMENTS_NAME,
    CATALOGS_NAME,
    HEAD_NAME,
];
const VERSION_TWO_MARKERS: [&str; 10] = [
    "reader.lock",
    "FORMAT",
    "FORMAT.next",
    "migration.intent",
    "migration.intent.next",
    "migration.receipt",
    "migration.receipt.next",
    "retention",
    "gc",
    "recovery",
];
const READER_LOCK_NAME: &str = "reader.lock";
const MARKER_NAME: &str = "FORMAT";
const INTENT_NAME: &str = "migration.intent";
const RECEIPT_NAME: &str = "migration.receipt";
const RETENTION_NAME: &str = "retention";
const GC_NAME: &str = "gc";
const RECOVERY_NAME: &str = "recovery";
const ROOTS_NAME: &str = "roots";
const MANIFESTS_NAME: &str = "manifests";
const DISPOSITIONS_NAME: &str = "dispositions";
const VERSION_TWO_NAMES: [&str; 12] = [
    LOCK_NAME,
    STAGING_NAME,
    SEGMENTS_NAME,
    CATALOGS_NAME,
    HEAD_NAME,
    READER_LOCK_NAME,
    MARKER_NAME,
    INTENT_NAME,
    RECEIPT_NAME,
    RETENTION_NAME,
    GC_NAME,
    RECOVERY_NAME,
];

pub(super) fn admit(directory: &Dir) -> io::Result<()> {
    admit_optional_file(directory, LOCK_NAME)?;
    admit_optional_directory(directory, STAGING_NAME)?;
    admit_optional_directory(directory, SEGMENTS_NAME)?;
    admit_optional_directory(directory, CATALOGS_NAME)?;
    admit_membership(directory, &INITIALIZATION_NAMES)
}

pub(super) fn admit_published(directory: &Dir) -> io::Result<()> {
    admit_required_file(directory, LOCK_NAME)?;
    admit_required_directory(directory, STAGING_NAME)?;
    admit_required_directory(directory, SEGMENTS_NAME)?;
    admit_required_directory(directory, CATALOGS_NAME)?;
    admit_required_file(directory, HEAD_NAME)?;
    admit_membership(directory, &PUBLISHED_NAMES)
}

/// Refuses version-two residue before version-one recovery touches a pool.
///
/// Recovery may open a store at any lawful point of its version-one lifecycle,
/// including before first publication and with `head.next` retained, and the
/// recovery inventory already classifies every unknown root entry as
/// unexpected. What that classification cannot express is that a root has left
/// version one entirely: a format marker, reader fence, migration record or
/// stage, or a `retention`, `gc`, or `recovery` directory means version-one
/// recovery must refuse before pinning anything.
pub(super) fn admit_recoverable(directory: &Dir) -> io::Result<()> {
    for entry in directory.entries()? {
        let name = entry?.file_name();
        if is_canonical(&name, &VERSION_TWO_MARKERS) {
            return Err(ambiguous_namespace());
        }
    }
    Ok(())
}

/// Admits the exact completely migrated version-2 root namespace.
///
/// Every version-1 published entry, the persistent reader fence, the format
/// marker, both migration records, and all three protocol directories must be
/// present. Any other entry is unrecoverable ambiguity.
pub(super) fn admit_version_two(directory: &Dir) -> io::Result<()> {
    admit_required_file(directory, LOCK_NAME)?;
    admit_required_directory(directory, STAGING_NAME)?;
    admit_required_directory(directory, SEGMENTS_NAME)?;
    admit_required_directory(directory, CATALOGS_NAME)?;
    admit_required_file(directory, HEAD_NAME)?;
    admit_required_file(directory, READER_LOCK_NAME)?;
    admit_required_file(directory, MARKER_NAME)?;
    admit_required_file(directory, INTENT_NAME)?;
    admit_required_file(directory, RECEIPT_NAME)?;
    admit_required_directory(directory, RETENTION_NAME)?;
    admit_required_directory(directory, GC_NAME)?;
    admit_required_directory(directory, RECOVERY_NAME)?;
    admit_membership(directory, &VERSION_TWO_NAMES)?;
    admit_version_two_protocol_directories(directory)
}

/// Admits the nested version-2 protocol directories the migration writer left.
///
/// `retention` must carry both immutable pools (its head and stages belong to
/// retention publication); `gc` must be empty until `KEEP-GC-001` implements
/// its records; `recovery` must hold exactly an empty `dispositions`. This is
/// the same membership `verify_prefix_directories` requires at the end of
/// migration, so a root that drifted after migration refuses here rather than
/// as a later pinning failure.
fn admit_version_two_protocol_directories(directory: &Dir) -> io::Result<()> {
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
    admit_optional_kind(directory, name, cap_std::fs::FileType::is_file)
}

fn admit_optional_directory(directory: &Dir, name: &str) -> io::Result<()> {
    admit_optional_kind(directory, name, cap_std::fs::FileType::is_dir)
}

fn admit_required_file(directory: &Dir, name: &str) -> io::Result<()> {
    admit_required_kind(directory, name, cap_std::fs::FileType::is_file)
}

fn admit_required_directory(directory: &Dir, name: &str) -> io::Result<()> {
    admit_required_kind(directory, name, cap_std::fs::FileType::is_dir)
}

fn admit_required_kind(
    directory: &Dir,
    name: &str,
    expected: fn(&cap_std::fs::FileType) -> bool,
) -> io::Result<()> {
    let metadata = directory.symlink_metadata(name)?;
    if expected(&metadata.file_type()) {
        Ok(())
    } else {
        Err(ambiguous_namespace())
    }
}

fn admit_optional_kind(
    directory: &Dir,
    name: &str,
    expected: fn(&cap_std::fs::FileType) -> bool,
) -> io::Result<()> {
    match directory.symlink_metadata(name) {
        Ok(metadata) if expected(&metadata.file_type()) => Ok(()),
        Ok(_) => Err(ambiguous_namespace()),
        Err(source) if source.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(source) => Err(source),
    }
}

fn admit_membership(directory: &Dir, canonical_names: &[&str]) -> io::Result<()> {
    let mut observed = 0_usize;
    for entry in directory.entries()? {
        observed = observed.checked_add(1).ok_or_else(ambiguous_namespace)?;
        if observed > canonical_names.len() {
            return Err(ambiguous_namespace());
        }
        let name = entry?.file_name();
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
    io::Error::new(
        io::ErrorKind::InvalidData,
        "store root is not an empty or partial canonical initialization namespace",
    )
}
