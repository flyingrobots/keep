//! This module owns semantic recovery-stage assessment failures.

use std::error::Error;
use std::fmt;

use super::{RecoveryCatalogStageError, RecoveryNextHeadStageError, RecoverySegmentStageError};

/// Why fingerprint-bound stage bytes had no lawful semantic assessment.
#[derive(Debug)]
pub enum RecoveryStageAssessmentError {
    /// Segment-stage classification failed.
    Segment {
        /// Exact segment-stage refusal.
        source: RecoverySegmentStageError,
    },
    /// Catalog-stage classification failed.
    Catalog {
        /// Exact catalog-stage refusal.
        source: RecoveryCatalogStageError,
    },
    /// Candidate-head classification failed.
    NextHead {
        /// Exact next-head refusal.
        source: RecoveryNextHeadStageError,
    },
}

impl fmt::Display for RecoveryStageAssessmentError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Segment { source } => {
                write!(formatter, "segment-stage assessment failed: {source}")
            }
            Self::Catalog { source } => {
                write!(formatter, "catalog-stage assessment failed: {source}")
            }
            Self::NextHead { source } => {
                write!(formatter, "next-head assessment failed: {source}")
            }
        }
    }
}

impl Error for RecoveryStageAssessmentError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Segment { source } => Some(source),
            Self::Catalog { source } => Some(source),
            Self::NextHead { source } => Some(source),
        }
    }
}
