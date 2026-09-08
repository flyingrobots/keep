//! This boundary module owns streamed migration inventory refusals.

use std::error::Error;
use std::fmt;

use super::{StoreMigrationInventoryEntry, StoreMigrationInventoryEntryCount};

/// Failure to stream one bounded canonical migration inventory.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StoreMigrationInventoryError {
    /// An entry would exceed the declared inventory count.
    EntryCountExceeded {
        /// Exact count declared before hashing.
        expected: StoreMigrationInventoryEntryCount,
        /// Count that the attempted entry would produce.
        observed: u32,
    },
    /// The same canonical entry appeared more than once.
    Duplicate {
        /// Repeated canonical entry.
        entry: StoreMigrationInventoryEntry,
    },
    /// Canonical entry order moved backward.
    OutOfOrder {
        /// Last entry admitted before the refusal.
        previous: StoreMigrationInventoryEntry,
        /// Entry observed after `previous`.
        observed: StoreMigrationInventoryEntry,
    },
    /// Finalization observed fewer entries than declared.
    Incomplete {
        /// Exact count declared before hashing.
        expected: StoreMigrationInventoryEntryCount,
        /// Exact number of entries admitted.
        observed: u32,
    },
    /// Checked observed-count arithmetic overflowed.
    EntryCountOverflow,
}

impl fmt::Display for StoreMigrationInventoryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EntryCountExceeded { expected, observed } => write!(
                formatter,
                "migration inventory expected {} entries but observed at least {observed}",
                expected.get()
            ),
            Self::Duplicate { .. } => formatter.write_str("duplicate migration inventory entry"),
            Self::OutOfOrder { .. } => {
                formatter.write_str("migration inventory entries are out of canonical order")
            }
            Self::Incomplete { expected, observed } => write!(
                formatter,
                "migration inventory expected {} entries but observed {observed}",
                expected.get()
            ),
            Self::EntryCountOverflow => {
                formatter.write_str("migration inventory entry count overflow")
            }
        }
    }
}

impl Error for StoreMigrationInventoryError {}
