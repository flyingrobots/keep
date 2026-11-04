//! This module owns exact protocol-entry spelling at the native lookup boundary.

use std::{ffi::OsStr, io};

use cap_std::fs::Dir;

/// Verifies an existing entry before a lookup may read, reuse, or mutate it.
/// Absence is permitted so create-new operations can retain their atomicity.
///
/// Case-insensitive APFS resolves `head` when asked for `HEAD`, and folds more
/// than ASCII case. The directory listing is the only observation of the stored
/// spelling: `F_GETPATH` reports a name cached on the vnode, which for Keep's
/// hard-link publication can belong to another link of the same file. So a name
/// the lookup resolves must also appear byte-for-byte in the listing. This
/// costs one streaming scan of the parent per existing entry, allocates one name
/// at a time, and has no durability effect.
#[cfg(target_os = "macos")]
pub(super) fn require_exact(directory: &Dir, name: &OsStr) -> io::Result<()> {
    match directory.symlink_metadata(name) {
        Err(source) if source.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(source) => return Err(source),
        Ok(_) => {}
    }
    for entry in directory.entries()? {
        if entry?.file_name() == name {
            return Ok(());
        }
    }
    Err(io::Error::new(
        io::ErrorKind::InvalidData,
        EntrySpellingMismatch {
            expected: name.to_owned(),
        },
    ))
}

// Production Linux admission excludes casefold directories. Other platforms
// have no production admission; their repository fixtures retain old behavior.
#[cfg(not(target_os = "macos"))]
pub(super) fn require_exact(_directory: &Dir, _name: &OsStr) -> io::Result<()> {
    Ok(())
}

/// The lookup resolved `expected`, but no directory entry has that spelling.
#[cfg(target_os = "macos")]
#[derive(Debug, Eq, PartialEq)]
pub(super) struct EntrySpellingMismatch {
    pub(super) expected: std::ffi::OsString,
}

#[cfg(target_os = "macos")]
impl std::fmt::Display for EntrySpellingMismatch {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "lookup resolved {} but no entry has that exact spelling",
            self.expected.display()
        )
    }
}

#[cfg(target_os = "macos")]
impl std::error::Error for EntrySpellingMismatch {}
