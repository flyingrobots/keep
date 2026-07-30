//! This module owns admitted recovery states for one segment stage.

use super::{AdmittedSegment, RecoverySegmentTruncation, RecoveryStageLength};

/// Semantic state of complete caller-supplied `current.seg` bytes.
#[must_use]
pub enum RecoverySegmentStage<'a> {
    /// Header plus zero or more complete admitted records, without a seal.
    Reusable(ReusableRecoverySegment),
    /// Fully admitted immutable segment, including its terminal seal.
    Complete(AdmittedSegment<'a>),
    /// Incomplete bytes whose exact missing boundary is known.
    Truncated(RecoverySegmentTruncation),
}

/// Validated reusable segment prefix retained for explicit recovery.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReusableRecoverySegment {
    record_count: u32,
    length: RecoveryStageLength,
}

impl ReusableRecoverySegment {
    pub(super) const fn new(record_count: u32, length: RecoveryStageLength) -> Self {
        Self {
            record_count,
            length,
        }
    }

    /// Returns the exact number of complete admitted records.
    #[must_use]
    pub const fn record_count(self) -> u32 {
        self.record_count
    }

    /// Returns the complete validated prefix length.
    pub const fn length(self) -> RecoveryStageLength {
        self.length
    }
}
