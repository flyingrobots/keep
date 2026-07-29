//! This module owns immutable recovery-stage evidence.

use super::{RecoveryStage, RecoveryStageFingerprint, RecoveryStageLength};

/// Fingerprint-bound evidence for one complete observed fixed stage.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RecoveryStageEvidence {
    stage: RecoveryStage,
    length: RecoveryStageLength,
    fingerprint: RecoveryStageFingerprint,
}

impl RecoveryStageEvidence {
    pub(super) const fn new(
        stage: RecoveryStage,
        length: RecoveryStageLength,
        fingerprint: RecoveryStageFingerprint,
    ) -> Self {
        Self {
            stage,
            length,
            fingerprint,
        }
    }

    /// Returns the exact fixed stage.
    #[must_use]
    pub const fn stage(self) -> RecoveryStage {
        self.stage
    }

    /// Returns the complete observed byte length.
    pub const fn length(self) -> RecoveryStageLength {
        self.length
    }

    /// Returns the domain-separated digest of the observed bytes.
    pub const fn fingerprint(self) -> RecoveryStageFingerprint {
        self.fingerprint
    }
}
