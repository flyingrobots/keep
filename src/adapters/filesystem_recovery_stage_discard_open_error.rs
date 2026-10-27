//! This module owns filesystem stage-discard authority acquisition failures.

use std::error::Error;
use std::fmt;
use std::io;

use super::RecoveryInventoryError;

/// Why a pinned writer-authorized stage discarder could not be opened.
#[derive(Debug)]
pub enum FilesystemRecoveryStageDiscardOpenError {
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

impl fmt::Display for FilesystemRecoveryStageDiscardOpenError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CloneRoot { source } => {
                write!(formatter, "locked recovery root clone failed: {source}")
            }
            Self::Namespace { source } => {
                write!(
                    formatter,
                    "recovery discard namespace was refused: {source}"
                )
            }
        }
    }
}

impl Error for FilesystemRecoveryStageDiscardOpenError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::CloneRoot { source } => Some(source),
            Self::Namespace { source } => Some(source),
        }
    }
}
