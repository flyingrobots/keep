//! This boundary module owns the digests a GC record computes over itself.

/// Canonical BLAKE3-256 digest of one exact candidate entry set.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct GcCandidateSetDigest([u8; 32]);

impl GcCandidateSetDigest {
    pub(super) const fn from_verified(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Returns the exact 32 digest bytes.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

/// Canonical BLAKE3-256 identity of one complete GC retirement intent.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct GcRetirementIntentDigest([u8; 32]);

impl GcRetirementIntentDigest {
    pub(super) const fn from_hash(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Returns the exact 32 digest bytes.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}
