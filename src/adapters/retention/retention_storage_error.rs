//! This module owns typed failures at the retention storage boundary.

use std::error::Error;
use std::fmt;
use std::io;

use super::RetentionRecordRefusal;

/// An operational failure or exact-record refusal during retention storage.
#[derive(Debug)]
#[non_exhaustive]
pub enum RetentionStorageError {
    /// A filesystem operation failed.
    Io {
        /// The original operational error, including its OS code and source.
        source: io::Error,
    },
    /// The record did not match its admitted evidence.
    Refused {
        /// The precise record refusal.
        source: RetentionRecordRefusal,
    },
}

impl fmt::Display for RetentionStorageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { .. } => formatter.write_str("retention storage operation failed"),
            Self::Refused { source } => fmt::Display::fmt(source, formatter),
        }
    }
}

impl Error for RetentionStorageError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io { source } => Some(source),
            Self::Refused { source } => Some(source),
        }
    }
}

impl From<io::Error> for RetentionStorageError {
    fn from(source: io::Error) -> Self {
        Self::Io { source }
    }
}

impl From<RetentionStorageError> for io::Error {
    fn from(error: RetentionStorageError) -> Self {
        match error {
            RetentionStorageError::Io { source } => source,
            refused @ RetentionStorageError::Refused { .. } => {
                Self::new(io::ErrorKind::InvalidData, refused)
            }
        }
    }
}
