//! This module owns fingerprint-bound materialized recovery-stage bytes.

use super::{RecoveryStage, RecoveryStageEvidence};

/// Complete materialized bytes proven equal to prior stage evidence.
#[must_use]
pub struct AdmittedRecoveryStageBytes<'a> {
    evidence: RecoveryStageEvidence,
    encoded: &'a [u8],
}

impl<'a> AdmittedRecoveryStageBytes<'a> {
    pub(super) const fn new(evidence: RecoveryStageEvidence, encoded: &'a [u8]) -> Self {
        Self { evidence, encoded }
    }

    /// Returns the canonical-name-selected fixed stage.
    #[must_use]
    pub const fn stage(&self) -> RecoveryStage {
        self.evidence.stage()
    }

    /// Returns the exact previously observed evidence.
    pub const fn evidence(&self) -> RecoveryStageEvidence {
        self.evidence
    }

    /// Returns the complete fingerprint-matched bytes.
    #[must_use]
    pub const fn encoded(&self) -> &'a [u8] {
        self.encoded
    }
}
