//! This module owns canonical retention root identity.

/// Canonical BLAKE3-256 identity of one complete retention root record.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RetentionRootDigest([u8; 32]);

impl RetentionRootDigest {
    pub(crate) const fn from_hash(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Returns the exact 32 digest bytes.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}
