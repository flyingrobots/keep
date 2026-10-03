//! This module owns demonstrated protocol namespace contradictions.

use cap_std::fs::{Dir, FileType};
use std::{error::Error, fmt, io};

/// No-follow kind observed for one protocol namespace entry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FilesystemEntryKind {
    /// A regular file.
    File,
    /// A directory.
    Directory,
    /// A symbolic link, regardless of its target.
    Symlink,
    /// Any other filesystem entry kind.
    Other,
}

impl FilesystemEntryKind {
    pub(super) fn observed(kind: FileType) -> Self {
        if kind.is_file() {
            Self::File
        } else if kind.is_dir() {
            Self::Directory
        } else if kind.is_symlink() {
            Self::Symlink
        } else {
            Self::Other
        }
    }
}

/// Observed evidence contradicts protocol membership or the required entry kind.
///
/// Operational metadata/open failures remain their original I/O causes. This
/// evidence does not make a subsequent pathname operation conditional on identity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum FilesystemNamespaceRefusal {
    /// An observed entry is outside the admitted protocol membership.
    UnexpectedEntry,
    /// A no-follow metadata observation established the wrong entry kind.
    WrongKind {
        /// Kind required by the protocol.
        expected: FilesystemEntryKind,
        /// Kind actually observed without following a symlink.
        observed: FilesystemEntryKind,
    },
}

impl fmt::Display for FilesystemNamespaceRefusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedEntry => formatter.write_str("unexpected protocol namespace entry"),
            Self::WrongKind { expected, observed } => write!(
                formatter,
                "protocol entry kind is {observed:?}, expected {expected:?}"
            ),
        }
    }
}

impl Error for FilesystemNamespaceRefusal {}

impl FilesystemNamespaceRefusal {
    pub(super) fn into_io(self) -> io::Error {
        io::Error::new(io::ErrorKind::InvalidData, self)
    }
}

pub(super) fn require_kind(actual: FileType, expected: FilesystemEntryKind) -> io::Result<()> {
    let observed = FilesystemEntryKind::observed(actual);
    if observed == expected {
        Ok(())
    } else {
        Err(FilesystemNamespaceRefusal::WrongKind { expected, observed }.into_io())
    }
}

pub(super) fn require_directory(parent: &Dir, name: &str) -> io::Result<()> {
    require_kind(
        parent.symlink_metadata(name)?.file_type(),
        FilesystemEntryKind::Directory,
    )
}
