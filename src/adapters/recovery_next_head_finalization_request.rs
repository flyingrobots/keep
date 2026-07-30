//! This module owns explicit requests to finalize one exact recovery next head.

use super::{
    CatalogPublicationExpectation, RecoveryNextHeadFinalizationTarget, RecoveryStageEvidence,
};

/// Authorized finalization of one complete `head.next` candidate.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RecoveryNextHeadFinalizationRequest {
    evidence: RecoveryStageEvidence,
    expectation: CatalogPublicationExpectation,
    target: RecoveryNextHeadFinalizationTarget,
}

impl RecoveryNextHeadFinalizationRequest {
    pub(super) const fn new(
        evidence: RecoveryStageEvidence,
        expectation: CatalogPublicationExpectation,
        target: RecoveryNextHeadFinalizationTarget,
    ) -> Self {
        Self {
            evidence,
            expectation,
            target,
        }
    }

    /// Returns the exact `head.next` evidence that authorized the request.
    pub const fn evidence(self) -> RecoveryStageEvidence {
        self.evidence
    }

    /// Returns the durable current-state expectation to revalidate.
    pub const fn expectation(self) -> CatalogPublicationExpectation {
        self.expectation
    }

    /// Returns the complete candidate catalog coordinate.
    pub const fn target(self) -> RecoveryNextHeadFinalizationTarget {
        self.target
    }
}
