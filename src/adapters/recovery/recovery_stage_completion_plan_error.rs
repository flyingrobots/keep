//! This module owns complete-stage recovery planning refusals.

use std::error::Error;
use std::fmt;

use super::RecoveryStage;

/// Why an assessed stage cannot enter immutable-pool completion.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecoveryStageCompletionPlanError {
    /// A segment or catalog stage is not semantically complete.
    NotComplete {
        /// Fixed stage that requires a different recovery action.
        stage: RecoveryStage,
    },
    /// The stage belongs to a different recovery protocol.
    NotPoolStage {
        /// Fixed stage that cannot name an immutable-pool artifact.
        stage: RecoveryStage,
    },
}

impl fmt::Display for RecoveryStageCompletionPlanError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotComplete { stage } => {
                write!(formatter, "{stage} is not a complete immutable artifact")
            }
            Self::NotPoolStage { stage } => {
                write!(formatter, "{stage} does not publish into an immutable pool")
            }
        }
    }
}

impl Error for RecoveryStageCompletionPlanError {}
