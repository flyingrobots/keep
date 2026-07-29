//! This module owns ordered complete-stage recovery failures.

use std::error::Error;
use std::fmt;
use std::io;

use super::{
    RecoveryStage, RecoveryStageCompletionPool, RecoveryStageCompletionStorageError,
    RecoveryStageCompletionTarget, RecoveryStageDiscardStorageError,
};

/// Exact failed phase of complete-stage recovery execution.
#[derive(Debug)]
pub enum RecoveryStageCompletionError {
    /// The exact present stage could not be verified and synchronized.
    SynchronizeStage {
        /// Fixed stage that could not be made durable.
        stage: RecoveryStage,
        /// Exact underlying verification or synchronization failure.
        source: RecoveryStageCompletionStorageError,
    },
    /// The exact stage could not be linked or an existing coordinate admitted.
    LinkOrAdmit {
        /// Validated immutable-pool target.
        target: RecoveryStageCompletionTarget,
        /// Exact underlying storage failure.
        source: RecoveryStageCompletionStorageError,
    },
    /// The immutable-pool entry did not verify exactly.
    VerifyPool {
        /// Validated immutable-pool target.
        target: RecoveryStageCompletionTarget,
        /// Exact underlying verification failure.
        source: RecoveryStageCompletionStorageError,
    },
    /// The immutable-pool directory could not be synchronized.
    SynchronizePool {
        /// Selected immutable pool.
        pool: RecoveryStageCompletionPool,
        /// Exact underlying synchronization failure.
        source: io::Error,
    },
    /// The exact stage could not be removed or admitted absent.
    RemoveStage {
        /// Exact semantic removal refusal.
        source: RecoveryStageDiscardStorageError,
    },
    /// The staging directory could not be synchronized after stage removal.
    SynchronizeStaging {
        /// Stage whose absence was not durably confirmed.
        stage: RecoveryStage,
        /// Exact underlying synchronization failure.
        source: io::Error,
    },
}

impl fmt::Display for RecoveryStageCompletionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SynchronizeStage { stage, source } => {
                write!(
                    formatter,
                    "failed to synchronize complete {stage}: {source}"
                )
            }
            Self::LinkOrAdmit { target, source } => write!(
                formatter,
                "failed to link or admit the {} recovery target: {source}",
                target.pool()
            ),
            Self::VerifyPool { target, source } => write!(
                formatter,
                "failed to verify the {} recovery target: {source}",
                target.pool()
            ),
            Self::SynchronizePool { pool, source } => {
                write!(formatter, "failed to synchronize the {pool} pool: {source}")
            }
            Self::RemoveStage { source } => {
                write!(formatter, "failed to remove the completed stage: {source}")
            }
            Self::SynchronizeStaging { stage, source } => write!(
                formatter,
                "failed to synchronize staging after {stage} removal: {source}"
            ),
        }
    }
}

impl Error for RecoveryStageCompletionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::SynchronizeStage { source, .. }
            | Self::LinkOrAdmit { source, .. }
            | Self::VerifyPool { source, .. } => Some(source),
            Self::SynchronizePool { source, .. } | Self::SynchronizeStaging { source, .. } => {
                Some(source)
            }
            Self::RemoveStage { source } => Some(source),
        }
    }
}
