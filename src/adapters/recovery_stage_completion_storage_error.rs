//! This module owns semantic storage refusals during stage completion.

use std::error::Error;
use std::fmt;
use std::io;

use super::{RecoveryStageCompletionRequest, RecoveryStageEvidence};

/// Why storage could not continue one exact complete-stage request.
#[derive(Debug)]
pub enum RecoveryStageCompletionStorageError {
    /// A canonical stage or pool entry resolves to different evidence.
    EvidenceMismatch {
        /// Evidence bound into the explicit completion request.
        expected: RecoveryStageEvidence,
        /// Evidence observed immediately before the refused transition.
        observed: RecoveryStageEvidence,
    },
    /// Neither the fixed stage nor its immutable-pool coordinate exists.
    Missing {
        /// Exact completion request that has no recoverable artifact.
        request: RecoveryStageCompletionRequest,
    },
    /// The storage boundary failed while observing or mutating.
    Storage {
        /// Exact underlying storage failure.
        source: io::Error,
    },
}

impl RecoveryStageCompletionStorageError {
    pub(super) const fn storage(source: io::Error) -> Self {
        Self::Storage { source }
    }
}

impl fmt::Display for RecoveryStageCompletionStorageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EvidenceMismatch { expected, observed } => write!(
                formatter,
                "{} completion evidence changed from length {} to length {}",
                expected.stage(),
                expected.length().get(),
                observed.length().get()
            ),
            Self::Missing { request } => write!(
                formatter,
                "{} and its {} pool coordinate are both absent",
                request.evidence().stage(),
                request.pool()
            ),
            Self::Storage { source } => {
                write!(formatter, "recovery stage completion failed: {source}")
            }
        }
    }
}

impl Error for RecoveryStageCompletionStorageError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Storage { source } => Some(source),
            Self::EvidenceMismatch { .. } | Self::Missing { .. } => None,
        }
    }
}
