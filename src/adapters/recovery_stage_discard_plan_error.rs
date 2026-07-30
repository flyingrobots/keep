//! This module owns truncated-stage discard-planning refusals.

use std::error::Error;
use std::fmt;

use super::RecoveryStage;

/// Why a semantic stage assessment cannot authorize truncated-stage discard.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecoveryStageDiscardPlanError {
    /// The stage is reusable or complete rather than exactly truncated.
    NotTruncated {
        /// Canonical fixed stage whose lawful state forbids this discard plan.
        stage: RecoveryStage,
    },
}

impl fmt::Display for RecoveryStageDiscardPlanError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotTruncated { stage } => {
                write!(formatter, "{stage} is not an exactly truncated stage")
            }
        }
    }
}

impl Error for RecoveryStageDiscardPlanError {}
