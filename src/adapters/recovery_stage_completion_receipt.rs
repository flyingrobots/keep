//! This module owns durable valid-orphan recovery receipts.

use super::{
    RecoveryStageCompletionRequest, RecoveryStageCompletionTarget, RecoveryStageDiscardOutcome,
    RecoveryStageEvidence, RecoveryStagePoolOutcome, RecoveryStageSynchronizationOutcome,
};

/// Proof that one exact artifact is pooled and its fixed stage is durably absent.
///
/// This receipt establishes a valid immutable orphan only. It makes no claim
/// that a catalog head names the artifact or that retention keeps it live.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RecoveryStageCompletionReceipt {
    request: RecoveryStageCompletionRequest,
    synchronization_outcome: RecoveryStageSynchronizationOutcome,
    pool_outcome: RecoveryStagePoolOutcome,
    stage_outcome: RecoveryStageDiscardOutcome,
}

impl RecoveryStageCompletionReceipt {
    pub(super) const fn new(
        request: RecoveryStageCompletionRequest,
        synchronization_outcome: RecoveryStageSynchronizationOutcome,
        pool_outcome: RecoveryStagePoolOutcome,
        stage_outcome: RecoveryStageDiscardOutcome,
    ) -> Self {
        Self {
            request,
            synchronization_outcome,
            pool_outcome,
            stage_outcome,
        }
    }

    /// Returns whether the exact stage was synchronized or already absent.
    pub const fn synchronization_outcome(self) -> RecoveryStageSynchronizationOutcome {
        self.synchronization_outcome
    }

    /// Returns the exact stage evidence bound into the request.
    pub const fn evidence(self) -> RecoveryStageEvidence {
        self.request.evidence()
    }

    /// Returns the verified immutable-pool coordinate.
    pub const fn target(self) -> RecoveryStageCompletionTarget {
        self.request.target()
    }

    /// Returns whether the pool entry was linked or already present.
    pub const fn pool_outcome(self) -> RecoveryStagePoolOutcome {
        self.pool_outcome
    }

    /// Returns whether the exact stage was removed or already absent.
    pub const fn stage_outcome(self) -> RecoveryStageDiscardOutcome {
        self.stage_outcome
    }
}
