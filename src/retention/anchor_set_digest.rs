//! This module owns one verified version-2 retention anchor-set digest.

/// BLAKE3-256 digest of one canonical retention anchor set.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RetentionAnchorSetDigest([u8; 32]);

impl RetentionAnchorSetDigest {
    /// Returns the exact digest bytes.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub(crate) const fn from_verified(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}
