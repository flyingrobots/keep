//! Typed physical immutable-segment digest.

/// BLAKE3-256 physical coordinate of one exact immutable segment prefix and
/// seal metadata.
///
/// This is not a logical content identity, authentication tag, publication
/// claim, or retention claim.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SegmentDigest([u8; 32]);

impl SegmentDigest {
    /// Returns the exact 32 digest bytes.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub(super) const fn from_validated(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}
