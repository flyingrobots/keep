//! This module owns semantic storage refusals during segment continuation.

use std::error::Error;
use std::fmt;
use std::io;

use super::{RecoverySegmentResumeRequest, RecoveryStageEvidence};

/// Why storage could not reopen one exact reusable segment prefix.
#[derive(Debug)]
pub enum RecoverySegmentResumeStorageError {
    /// The fixed stage resolves to different evidence.
    EvidenceMismatch {
        /// Evidence bound into the explicit continuation request.
        expected: RecoveryStageEvidence,
        /// Evidence observed while reopening the writable stage.
        observed: RecoveryStageEvidence,
    },
    /// The fixed segment stage is absent.
    Missing {
        /// Exact continuation request whose stage is absent.
        request: RecoverySegmentResumeRequest,
    },
    /// The storage boundary failed while reopening or materializing.
    Storage {
        /// Exact underlying storage failure.
        source: io::Error,
    },
}

impl RecoverySegmentResumeStorageError {
    /// Wraps an underlying storage failure without discarding its source.
    pub const fn storage(source: io::Error) -> Self {
        Self::Storage { source }
    }
}

impl fmt::Display for RecoverySegmentResumeStorageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EvidenceMismatch { expected, observed } => write!(
                formatter,
                "{} continuation evidence changed from length {} to length {}",
                expected.stage(),
                expected.length().get(),
                observed.length().get()
            ),
            Self::Missing { request } => {
                write!(formatter, "{} is absent", request.evidence().stage())
            }
            Self::Storage { source } => {
                write!(formatter, "recovery segment continuation failed: {source}")
            }
        }
    }
}

impl Error for RecoverySegmentResumeStorageError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Storage { source } => Some(source),
            Self::EvidenceMismatch { .. } | Self::Missing { .. } => None,
        }
    }
}
