//! This module owns the typed reasons initialization refuses a root.

use std::error::Error;
use std::fmt;
use std::path::PathBuf;

use crate::adapters::FilesystemEntryKind;

/// Why a root is not a recoverable initialization prefix.
///
/// Every refusal is returned before the refused root is mutated: the first
/// classification runs before `writer.lock` is created, and the second runs
/// under writer authority before any later phase.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum VersionTwoInitializationRefusal {
    /// The root holds `FORMAT` without `FORMAT.next`: initialization has
    /// already committed. Reopen the store instead.
    AlreadyInitialized,
    /// An entry that no initialization prefix contains at this point: an
    /// unknown name, a protocol entry without its predecessors, or a
    /// directory out of creation order.
    NotAPrefix {
        /// The entry's path relative to the store root.
        entry: PathBuf,
    },
    /// A canonical entry has the wrong kind.
    WrongKind {
        /// The entry's path relative to the store root.
        entry: &'static str,
        /// The required kind.
        expected: FilesystemEntryKind,
        /// The kind observed without following links.
        observed: FilesystemEntryKind,
    },
    /// `writer.lock` or `reader.lock` is not zero-length.
    NonEmptyLockFile {
        /// The lock file's name.
        entry: &'static str,
        /// The observed length.
        observed_length: u64,
    },
    /// `FORMAT.next` is longer than the marker or is not a byte prefix of it.
    MarkerStageMismatch {
        /// The canonical marker length.
        expected_length: usize,
        /// The observed stage length.
        observed_length: u64,
    },
    /// `FORMAT` beside `FORMAT.next` is not the same inode with the same
    /// canonical bytes.
    MarkerLinkMismatch,
}

impl fmt::Display for VersionTwoInitializationRefusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AlreadyInitialized => formatter.write_str("the store is already initialized"),
            Self::NotAPrefix { entry } => write!(
                formatter,
                "{} is not part of any initialization prefix",
                entry.display()
            ),
            Self::WrongKind {
                entry,
                expected,
                observed,
            } => write!(formatter, "{entry} is {observed:?}, expected {expected:?}"),
            Self::NonEmptyLockFile {
                entry,
                observed_length,
            } => write!(
                formatter,
                "{entry} holds {observed_length} bytes, expected 0"
            ),
            Self::MarkerStageMismatch {
                expected_length,
                observed_length,
            } => write!(
                formatter,
                "FORMAT.next holds {observed_length} bytes that are not a prefix of the \
                 {expected_length}-byte canonical marker"
            ),
            Self::MarkerLinkMismatch => {
                formatter.write_str("FORMAT and FORMAT.next are not one canonical marker inode")
            }
        }
    }
}

impl Error for VersionTwoInitializationRefusal {}
