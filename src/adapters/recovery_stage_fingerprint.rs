//! This module owns domain-separated recovery-stage fingerprints.

use super::RecoveryStageFingerprintAlgorithm;

/// Exact digest of one bounded observed recovery stage.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RecoveryStageFingerprint([u8; 32]);

impl RecoveryStageFingerprint {
    pub(super) const fn from_validated(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Returns the version-1 algorithm coordinate.
    pub const fn algorithm(self) -> RecoveryStageFingerprintAlgorithm {
        RecoveryStageFingerprintAlgorithm::FramedBlake3V1
    }

    /// Returns the exact 32-byte digest.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}
