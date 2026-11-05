//! This module owns filesystem segment-continuation authority failures.

use std::error::Error;
use std::fmt;
use std::io;

use super::{FilesystemRecoveryStageDiscardOpenError, RecoveryInventoryError};

/// Why a pinned writer-authorized segment resumer could not be opened.
#[derive(Debug)]
pub enum FilesystemRecoverySegmentResumeOpenError {
    /// The locked root capability could not be cloned for recovery inventory.
    CloneRoot {
        /// Exact root-capability clone failure.
        source: io::Error,
    },
    /// One pinned protocol namespace could not be admitted.
    Namespace {
        /// Exact recovery-namespace admission refusal.
        source: RecoveryInventoryError,
    },
}

impl From<FilesystemRecoveryStageDiscardOpenError> for FilesystemRecoverySegmentResumeOpenError {
    fn from(source: FilesystemRecoveryStageDiscardOpenError) -> Self {
        match source {
            FilesystemRecoveryStageDiscardOpenError::CloneRoot { source } => {
                Self::CloneRoot { source }
            }
            FilesystemRecoveryStageDiscardOpenError::Namespace { source } => {
                Self::Namespace { source }
            }
        }
    }
}

impl fmt::Display for FilesystemRecoverySegmentResumeOpenError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CloneRoot { source } => {
                write!(formatter, "locked recovery root clone failed: {source}")
            }
            Self::Namespace { source } => write!(
                formatter,
                "segment continuation namespace was refused: {source}"
            ),
        }
    }
}

impl Error for FilesystemRecoverySegmentResumeOpenError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::CloneRoot { source } => Some(source),
            Self::Namespace { source } => Some(source),
        }
    }
}
