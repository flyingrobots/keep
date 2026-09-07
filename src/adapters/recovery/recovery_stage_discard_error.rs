//! This module owns ordered truncated-stage discard execution failures.

use std::error::Error;
use std::fmt;
use std::io;

use super::{RecoveryStage, RecoveryStageDiscardStorageError};

/// Why explicit truncated-stage discard did not return a durable receipt.
#[derive(Debug)]
pub enum RecoveryStageDiscardError {
    /// Exact-evidence removal or absence admission failed.
    Remove {
        /// Exact storage refusal.
        source: RecoveryStageDiscardStorageError,
    },
    /// Synchronizing the name-selected parent directory failed.
    Synchronize {
        /// Canonical stage selecting `staging` or the store root.
        stage: RecoveryStage,
        /// Exact parent-directory synchronization failure.
        source: io::Error,
    },
}

impl fmt::Display for RecoveryStageDiscardError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Remove { source } => write!(formatter, "stage discard was refused: {source}"),
            Self::Synchronize { stage, source } => {
                write!(formatter, "{stage} parent synchronization failed: {source}")
            }
        }
    }
}

impl Error for RecoveryStageDiscardError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Remove { source } => Some(source),
            Self::Synchronize { source, .. } => Some(source),
        }
    }
}
