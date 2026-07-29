//! This module owns one exact reusable-segment continuation request.

use super::{RecoveryStageEvidence, RecoveryStageLength, SegmentReadPolicy, SegmentRecordLimit};

/// Owned authority to continue one exact validated reusable segment prefix.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[must_use]
pub struct RecoverySegmentResumeRequest {
    evidence: RecoveryStageEvidence,
    record_count: u32,
    length: RecoveryStageLength,
    policy: SegmentReadPolicy,
}

impl RecoverySegmentResumeRequest {
    pub(super) const fn new(
        evidence: RecoveryStageEvidence,
        record_count: u32,
        length: RecoveryStageLength,
        policy: SegmentReadPolicy,
    ) -> Self {
        Self {
            evidence,
            record_count,
            length,
            policy,
        }
    }

    /// Returns the exact prior observation that must still match.
    pub const fn evidence(self) -> RecoveryStageEvidence {
        self.evidence
    }

    /// Returns the number of complete records already admitted.
    pub const fn record_count(self) -> u32 {
        self.record_count
    }

    /// Returns the exact byte boundary after the admitted record prefix.
    pub const fn length(self) -> RecoveryStageLength {
        self.length
    }

    /// Returns the resource policy that governs re-admission and continuation.
    pub const fn policy(self) -> SegmentReadPolicy {
        self.policy
    }

    /// Returns the maximum complete-record count after continuation.
    pub const fn record_limit(self) -> SegmentRecordLimit {
        self.policy.record_limit()
    }
}
