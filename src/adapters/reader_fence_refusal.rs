//! This module owns the refusals of the persistent `reader.lock` fence shared
//! by retention snapshots and store migration.

use std::error::Error;
use std::fmt;

use super::{FilesystemEntryIdentity, FilesystemEntryKind};

/// Why an opened `reader.lock` handle and its named entry cannot fence one view.
///
/// Carried as the `InvalidData` payload of the fence's `io::Error`. The checks
/// run in this order: kind, then length, then identity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ReaderFenceRefusal {
    /// The opened handle or the named entry is not a regular file.
    Kind {
        /// Kind of the opened handle.
        observed_handle: FilesystemEntryKind,
        /// Kind of the named entry, observed without following a link.
        observed_entry: FilesystemEntryKind,
    },
    /// The fence is not empty; the protocol requires zero bytes.
    Length {
        /// Length of the opened handle.
        observed_handle: u64,
        /// Length of the named entry.
        observed_entry: u64,
    },
    /// The name no longer identifies the opened fence handle.
    Identity {
        /// Identity of the opened handle.
        expected: FilesystemEntryIdentity,
        /// Identity the name currently resolves to.
        observed: FilesystemEntryIdentity,
    },
}

impl fmt::Display for ReaderFenceRefusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Kind { .. } => "reader fence is not a regular file",
            Self::Length { .. } => "reader fence is not empty",
            Self::Identity { .. } => "reader fence changed identity",
        })
    }
}

impl Error for ReaderFenceRefusal {}
