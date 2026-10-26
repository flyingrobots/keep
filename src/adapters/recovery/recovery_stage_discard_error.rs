//! This module owns ordered truncated-stage discard execution failures.

use std::error::Error;
use std::fmt;
use std::io;

use super::{RecoveryStage, RecoveryStageDiscardOutcome, RecoveryStageDiscardStorageError};

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
        /// Removal or prior absence established before the failed sync.
        /// This is not a durable receipt; directory durability is unconfirmed.
        outcome: RecoveryStageDiscardOutcome,
        /// Exact parent-directory synchronization failure.
        source: io::Error,
    },
}

impl fmt::Display for RecoveryStageDiscardError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Remove { source } => write!(formatter, "stage discard was refused: {source}"),
            Self::Synchronize {
                stage,
                outcome,
                source,
            } => {
                write!(
                    formatter,
                    "{stage} parent synchronization failed after {outcome:?} (durability unconfirmed): {source}"
                )
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
