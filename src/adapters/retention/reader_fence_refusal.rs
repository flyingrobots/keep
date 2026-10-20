//! This module owns precise reader-fence kind, length, and identity refusals.

use std::error::Error;
use std::fmt;

/// The file kind observed at the reader fence boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReaderFenceKind {
    /// A regular file, as the protocol requires.
    RegularFile,
    /// A directory, symlink, device, or other non-regular entry.
    Other,
}

/// Why the opened handle and named fence cannot protect one reader view.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ReaderFenceRefusal {
    /// The fence handle or current entry is not a regular file.
    Kind {
        /// The protocol's required kind.
        expected: ReaderFenceKind,
        /// The opened handle's kind.
        observed_handle: ReaderFenceKind,
        /// The current entry's kind.
        observed_entry: ReaderFenceKind,
    },
    /// The fence is not empty.
    Length {
        /// The required zero length.
        expected: u64,
        /// The opened handle's length.
        observed_handle: u64,
        /// The current entry's length.
        observed_entry: u64,
    },
    /// The name no longer identifies the opened fence handle.
    Identity {
        /// The opened handle's device and inode.
        expected: (u64, u64),
        /// The current entry's device and inode.
        observed: (u64, u64),
    },
}

impl fmt::Display for ReaderFenceRefusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Kind { .. } => formatter.write_str("reader fence is not a regular file"),
            Self::Length {
                observed_handle,
                observed_entry,
                ..
            } => write!(
                formatter,
                "reader fence must be empty: handle length {observed_handle}, entry length {observed_entry}"
            ),
            Self::Identity { expected, observed } => write!(
                formatter,
                "reader fence changed identity: expected {expected:?}, observed {observed:?}"
            ),
        }
    }
}

impl Error for ReaderFenceRefusal {}
