//! This module owns durable recovery head-finalization receipts.

use super::{
    RecoveryNextHeadFinalizationOutcome, RecoveryNextHeadFinalizationRequest,
    RecoveryNextHeadFinalizationTarget, RecoveryStageEvidence,
};

/// Proof that durable `HEAD` names one exact candidate after root synchronization.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RecoveryNextHeadFinalizationReceipt {
    request: RecoveryNextHeadFinalizationRequest,
    outcome: RecoveryNextHeadFinalizationOutcome,
}

impl RecoveryNextHeadFinalizationReceipt {
    pub(super) const fn new(
        request: RecoveryNextHeadFinalizationRequest,
        outcome: RecoveryNextHeadFinalizationOutcome,
    ) -> Self {
        Self { request, outcome }
    }

    /// Returns the exact `head.next` evidence bound into the request.
    pub const fn evidence(self) -> RecoveryStageEvidence {
        self.request.evidence()
    }

    /// Returns the durable candidate catalog coordinate.
    pub const fn target(self) -> RecoveryNextHeadFinalizationTarget {
        self.request.target()
    }

    /// Returns whether this execution replaced or re-admitted durable `HEAD`.
    pub const fn outcome(self) -> RecoveryNextHeadFinalizationOutcome {
        self.outcome
    }
}
