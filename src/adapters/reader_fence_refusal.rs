//! This module owns the admission law and refusals of the persistent
//! `reader.lock` fence shared by retention snapshots and store migration.

use std::error::Error;
use std::fmt;
use std::io;

use cap_std::fs::Metadata;

use super::filesystem_exact_record::EntryIdentity;
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

impl ReaderFenceRefusal {
    /// Admits one fence observation: an opened handle and its no-follow named entry.
    pub(super) fn admit(handle: &Metadata, entry: &Metadata) -> io::Result<()> {
        let kind = |metadata: &Metadata| FilesystemEntryKind::observed(metadata.file_type());
        if !handle.is_file() || !entry.is_file() {
            return Err(Self::Kind {
                observed_handle: kind(handle),
                observed_entry: kind(entry),
            }
            .into_io());
        }
        if handle.len() != 0 || entry.len() != 0 {
            return Err(Self::Length {
                observed_handle: handle.len(),
                observed_entry: entry.len(),
            }
            .into_io());
        }
        let expected = EntryIdentity::from(handle);
        let observed = EntryIdentity::from(entry);
        if expected == observed {
            Ok(())
        } else {
            Err(Self::Identity {
                expected: expected.into(),
                observed: observed.into(),
            }
            .into_io())
        }
    }

    fn into_io(self) -> io::Error {
        io::Error::new(io::ErrorKind::InvalidData, self)
    }
}
