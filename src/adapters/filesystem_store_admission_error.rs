//! This module owns published-store platform-admission failures.

use std::error::Error;
use std::fmt;
use std::io;

use super::WriterLockAcquireError;

/// Failure to reacquire writer authority over one published filesystem store.
#[derive(Debug)]
#[non_exhaustive]
pub enum FilesystemStoreAdmissionError {
    /// The store root does not satisfy the production platform profile.
    Platform {
        /// Preserved platform-admission failure.
        source: io::Error,
    },
    /// Exclusive writer authority could not be acquired.
    WriterLock {
        /// Preserved writer-lock failure.
        source: WriterLockAcquireError,
    },
    /// The writer-locked published namespace is incomplete or ambiguous.
    Namespace {
        /// Preserved namespace-admission failure.
        source: io::Error,
    },
    /// The `FORMAT` marker failed exact admission.
    FormatMarker {
        /// Preserved record-admission failure.
        source: io::Error,
    },
}

impl fmt::Display for FilesystemStoreAdmissionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Platform { .. } => "published store platform admission failed",
            Self::WriterLock { .. } => "published store writer-lock acquisition failed",
            Self::Namespace { .. } => "published store namespace admission failed",
            Self::FormatMarker { .. } => "format marker admission failed",
        })
    }
}

impl Error for FilesystemStoreAdmissionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Platform { source }
            | Self::Namespace { source }
            | Self::FormatMarker { source } => Some(source),
            Self::WriterLock { source } => Some(source),
        }
    }
}
