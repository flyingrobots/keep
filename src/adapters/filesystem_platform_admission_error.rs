//! This module owns published-store platform-admission failures.

use std::error::Error;
use std::fmt;
use std::io;

use super::WriterLockAcquireError;

/// Failure to reacquire writer authority over one published filesystem store.
#[derive(Debug)]
pub enum FilesystemPlatformAdmissionError {
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
}

impl fmt::Display for FilesystemPlatformAdmissionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Platform { .. } => "published store platform admission failed",
            Self::WriterLock { .. } => "published store writer-lock acquisition failed",
            Self::Namespace { .. } => "published store namespace admission failed",
        })
    }
}

impl Error for FilesystemPlatformAdmissionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Platform { source } | Self::Namespace { source } => Some(source),
            Self::WriterLock { source } => Some(source),
        }
    }
}
