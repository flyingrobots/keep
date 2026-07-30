//! This module owns one verified version-2 retention-closure digest.

/// BLAKE3-256 digest of one canonical verified closure transcript.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RetentionClosureDigest([u8; 32]);

impl RetentionClosureDigest {
    /// Returns the exact digest bytes.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub(crate) const fn from_verified(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}
