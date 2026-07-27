//! Registered storage-profile admission failures.

use std::error::Error;
use std::fmt;

use super::StorageProfileId;

/// Failure to admit a canonical storage-profile identity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StorageProfileAdmissionError {
    /// The identity is canonical but this Keep version does not implement it.
    Unsupported {
        /// Canonical identity that was refused.
        observed: StorageProfileId,
    },
}

impl fmt::Display for StorageProfileAdmissionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unsupported { observed } => {
                write!(formatter, "unsupported storage profile {observed}")
            }
        }
    }
}

impl Error for StorageProfileAdmissionError {}
