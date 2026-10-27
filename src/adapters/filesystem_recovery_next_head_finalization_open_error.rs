//! This module owns filesystem next-head authority acquisition failures.

use std::error::Error;
use std::fmt;
use std::io;

use super::{FilesystemRecoveryStageDiscardOpenError, RecoveryInventoryError};

/// Why a pinned writer-authorized next-head finalizer could not be opened.
#[derive(Debug)]
pub enum FilesystemRecoveryNextHeadFinalizationOpenError {
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

impl From<FilesystemRecoveryStageDiscardOpenError>
    for FilesystemRecoveryNextHeadFinalizationOpenError
{
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

impl fmt::Display for FilesystemRecoveryNextHeadFinalizationOpenError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CloneRoot { source } => {
                write!(formatter, "locked recovery root clone failed: {source}")
            }
            Self::Namespace { source } => write!(
                formatter,
                "recovery next-head namespace was refused: {source}"
            ),
        }
    }
}

impl Error for FilesystemRecoveryNextHeadFinalizationOpenError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::CloneRoot { source } => Some(source),
            Self::Namespace { source } => Some(source),
        }
    }
}
