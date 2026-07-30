//! This module owns durable truncated-stage discard receipts.

use super::{
    RecoveryStage, RecoveryStageDiscardOutcome, RecoveryStageDiscardReason,
    RecoveryStageDiscardRequest, RecoveryStageEvidence,
};

/// Proof that exact stage removal or absence was followed by parent sync.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RecoveryStageDiscardReceipt {
    request: RecoveryStageDiscardRequest,
    outcome: RecoveryStageDiscardOutcome,
}

impl RecoveryStageDiscardReceipt {
    pub(super) const fn new(
        request: RecoveryStageDiscardRequest,
        outcome: RecoveryStageDiscardOutcome,
    ) -> Self {
        Self { request, outcome }
    }

    /// Returns the canonical fixed stage whose absence is durable.
    #[must_use]
    pub const fn stage(self) -> RecoveryStage {
        self.request.stage()
    }

    /// Returns the exact evidence bound into the discard request.
    pub const fn evidence(self) -> RecoveryStageEvidence {
        self.request.evidence()
    }

    /// Returns the exact truncation that authorized discard.
    pub const fn reason(self) -> RecoveryStageDiscardReason {
        self.request.reason()
    }

    /// Returns whether execution removed the stage or admitted prior absence.
    pub const fn outcome(self) -> RecoveryStageDiscardOutcome {
        self.outcome
    }
}
