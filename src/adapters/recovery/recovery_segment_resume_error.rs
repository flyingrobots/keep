//! This module owns reusable-segment continuation execution failures.

use std::error::Error;
use std::fmt;

use super::{
    RecoverySegmentResumeStorageError, RecoveryStageAssessmentError,
    RecoveryStageByteAdmissionError, SegmentReadError,
};

/// Why one reusable segment prefix could not become writable again.
#[derive(Debug)]
pub enum RecoverySegmentResumeError {
    /// Storage could not reopen the exact stage.
    Open {
        /// Exact storage refusal.
        source: RecoverySegmentResumeStorageError,
    },
    /// Materialized bytes no longer match prior evidence.
    Admission {
        /// Exact evidence-admission refusal.
        source: RecoveryStageByteAdmissionError,
    },
    /// Reopened bytes no longer admit the segment grammar.
    Assessment {
        /// Exact semantic assessment refusal.
        source: RecoveryStageAssessmentError,
    },
    /// Reopened bytes no longer classify as a reusable prefix.
    NotReusable,
    /// Rebuilding append state from re-admitted records failed.
    Rebuild {
        /// Exact record-cursor or allocation refusal.
        source: SegmentReadError,
    },
}

impl fmt::Display for RecoverySegmentResumeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Open { source } => write!(formatter, "cannot reopen reusable segment: {source}"),
            Self::Admission { source } => {
                write!(formatter, "reopened segment evidence disagrees: {source}")
            }
            Self::Assessment { source } => {
                write!(formatter, "reopened segment is invalid: {source}")
            }
            Self::NotReusable => write!(formatter, "reopened segment is no longer reusable"),
            Self::Rebuild { source } => {
                write!(formatter, "cannot rebuild reusable segment state: {source}")
            }
        }
    }
}

impl Error for RecoverySegmentResumeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Open { source } => Some(source),
            Self::Admission { source } => Some(source),
            Self::Assessment { source } => Some(source),
            Self::Rebuild { source } => Some(source),
            Self::NotReusable => None,
        }
    }
}
