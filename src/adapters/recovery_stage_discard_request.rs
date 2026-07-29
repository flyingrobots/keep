//! This module owns explicit fingerprint-bound stage-discard requests.

use super::{RecoveryStage, RecoveryStageDiscardReason, RecoveryStageEvidence};

/// Immutable request to discard one exactly observed truncated fixed stage.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RecoveryStageDiscardRequest {
    evidence: RecoveryStageEvidence,
    reason: RecoveryStageDiscardReason,
}

impl RecoveryStageDiscardRequest {
    pub(super) const fn new(
        evidence: RecoveryStageEvidence,
        reason: RecoveryStageDiscardReason,
    ) -> Self {
        Self { evidence, reason }
    }

    /// Returns the canonical fixed stage selected by the request.
    #[must_use]
    pub const fn stage(self) -> RecoveryStage {
        self.evidence.stage()
    }

    /// Returns the exact length and fingerprint that mutation must reverify.
    pub const fn evidence(self) -> RecoveryStageEvidence {
        self.evidence
    }

    /// Returns the exact truncation that authorized explicit discard.
    pub const fn reason(self) -> RecoveryStageDiscardReason {
        self.reason
    }
}
