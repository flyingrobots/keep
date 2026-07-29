//! This module owns bounded admission of the initialization namespace.

use std::ffi::OsStr;
use std::io;

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
