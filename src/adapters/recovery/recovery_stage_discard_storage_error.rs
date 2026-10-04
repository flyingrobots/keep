//! This module owns semantic storage refusals during stage removal.

use std::error::Error;
use std::fmt;
use std::io;

use super::RecoveryStageEvidence;

/// Why storage could not remove one exact fingerprint-bound stage.
#[derive(Debug)]
pub enum RecoveryStageDiscardStorageError {
    /// The canonical stage name resolves to different evidence.
    EvidenceMismatch {
        /// Evidence bound into the explicit discard request.
        expected: RecoveryStageEvidence,
        /// Evidence observed immediately before the refused mutation.
        observed: RecoveryStageEvidence,
    },
    /// The storage boundary failed while reopening, verifying, or removing.
    Storage {
        /// Exact underlying storage failure.
        source: io::Error,
    },
}

impl fmt::Display for RecoveryStageDiscardStorageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EvidenceMismatch { expected, observed } => write!(
                formatter,
                "{} evidence changed from length {} to length {}",
                expected.stage(),
                expected.length().get(),
                observed.length().get()
            ),
            Self::Storage { source } => {
                write!(formatter, "recovery stage removal failed: {source}")
            }
        }
    }
}

impl Error for RecoveryStageDiscardStorageError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Storage { source } => Some(source),
            Self::EvidenceMismatch { .. } => None,
        }
    }
}
