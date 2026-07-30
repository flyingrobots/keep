//! This boundary module owns migration inventory entry-count refusals.

use std::error::Error;
use std::fmt;

/// Failure to admit a bounded migration inventory entry count.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StoreMigrationInventoryEntryCountError {
    /// The requested count exceeds the immutable protocol maximum.
    AboveMaximum {
        /// Count supplied by the caller.
        observed: u32,
        /// Largest count admitted by the protocol.
        maximum: u32,
    },
}

impl fmt::Display for StoreMigrationInventoryEntryCountError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AboveMaximum { observed, maximum } => write!(
                formatter,
                "migration inventory entry count {observed} exceeds maximum {maximum}"
            ),
        }
    }
}

impl Error for StoreMigrationInventoryEntryCountError {}
