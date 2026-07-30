//! This module owns explicit fingerprint-bound stage-completion requests.

use super::{RecoveryStageCompletionPool, RecoveryStageCompletionTarget, RecoveryStageEvidence};

/// Authorized completion of one exact stage into one immutable pool.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RecoveryStageCompletionRequest {
    evidence: RecoveryStageEvidence,
    target: RecoveryStageCompletionTarget,
}

impl RecoveryStageCompletionRequest {
    pub(super) const fn new(
        evidence: RecoveryStageEvidence,
        target: RecoveryStageCompletionTarget,
    ) -> Self {
        Self { evidence, target }
    }

    /// Returns the exact stage evidence that authorized the request.
    pub const fn evidence(self) -> RecoveryStageEvidence {
        self.evidence
    }

    /// Returns the verified immutable-pool coordinate.
    pub const fn target(self) -> RecoveryStageCompletionTarget {
        self.target
    }

    /// Returns the immutable pool selected by the target.
    pub const fn pool(self) -> RecoveryStageCompletionPool {
        self.target.pool()
    }
}
