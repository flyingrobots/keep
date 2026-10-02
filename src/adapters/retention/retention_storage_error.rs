//! This module owns typed failures at the retention storage boundary.

use std::error::Error;
use std::fmt;
use std::io;

use super::{RetentionRecordRefusal, RetentionStorageProgress};

/// An operational failure or exact-record refusal during retention storage.
#[derive(Debug)]
#[non_exhaustive]
pub enum RetentionStorageError {
    /// An operation stopped at a known boundary, possibly after namespace effects.
    Operation {
        /// The original typed storage cause, never stringified.
        source: Box<RetentionStorageError>,
        /// Known effects and uncertainty within this failing capability.
        progress: RetentionStorageProgress,
    },
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
            Self::Operation { progress, .. } => write!(
                formatter,
                "retention storage stopped at {:?}; known effects {:?}, uncertain effect {:?}",
                progress.boundary, progress.known, progress.uncertain
            ),
            Self::Io { .. } => formatter.write_str("retention storage operation failed"),
            Self::Refused { source } => fmt::Display::fmt(source, formatter),
        }
    }
}

impl Error for RetentionStorageError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Operation { source, .. } => Some(source.as_ref()),
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
            refused @ (RetentionStorageError::Refused { .. }
            | RetentionStorageError::Operation { .. }) => {
                Self::new(io::ErrorKind::InvalidData, refused)
            }
        }
    }
}

impl RetentionStorageError {
    /// Progress within the failing capability, if its adapter reports it.
    ///
    /// `None` means unreported effects, not absence of effects. Callers must reobserve.
    #[must_use]
    pub const fn progress(&self) -> Option<&RetentionStorageProgress> {
        match self {
            Self::Operation { progress, .. } => Some(progress),
            _ => None,
        }
    }
}
