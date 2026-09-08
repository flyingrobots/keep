//! This module owns reusable-segment continuation planning refusals.

use std::error::Error;
use std::fmt;

use super::{RecoveryStage, SegmentRecordLimit};

/// Why an assessed stage cannot enter reusable-segment continuation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecoverySegmentResumePlanError {
    /// A segment stage is not a reusable complete-record prefix.
    NotReusable {
        /// Fixed stage that requires a different recovery action.
        stage: RecoveryStage,
    },
    /// The stage belongs to a different recovery protocol.
    NotSegment {
        /// Fixed stage that cannot be resumed as a segment.
        stage: RecoveryStage,
    },
    /// The selected continuation policy is below the admitted record count.
    RecordLimit {
        /// Maximum complete-record count allowed by the policy.
        maximum: SegmentRecordLimit,
        /// Complete records already present in the reusable prefix.
        observed: u32,
    },
}

impl fmt::Display for RecoverySegmentResumePlanError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotReusable { stage } => {
                write!(formatter, "{stage} is not a reusable segment prefix")
            }
            Self::NotSegment { stage } => {
                write!(formatter, "{stage} cannot be resumed as a segment")
            }
            Self::RecordLimit { maximum, observed } => write!(
                formatter,
                "reusable segment has {observed} records, above continuation limit {}",
                maximum.get()
            ),
        }
    }
}

impl Error for RecoverySegmentResumePlanError {}
