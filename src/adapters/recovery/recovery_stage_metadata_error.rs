//! This module owns recovery-stage metadata admission failures.

use std::error::Error;
use std::fmt;

use super::RecoveryStage;

/// Why fixed recovery-stage metadata could not be admitted.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecoveryStageMetadataError {
    /// Metadata reports a stage above its name-selected maximum.
    Oversized {
        /// Fixed stage being observed.
        stage: RecoveryStage,
        /// Name-selected maximum.
        maximum: u64,
        /// Metadata byte length.
        observed: u64,
    },
}

impl fmt::Display for RecoveryStageMetadataError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Oversized {
                stage,
                maximum,
                observed,
            } => write!(
                formatter,
                "{stage} metadata length {observed} exceeds maximum {maximum}"
            ),
        }
    }
}

impl Error for RecoveryStageMetadataError {}
