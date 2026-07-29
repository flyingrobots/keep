//! Verified physical catalog-generation digest.

/// Verified digest of one exact immutable catalog generation.
///
/// This value is a physical generation coordinate and predecessor witness. It
/// is not a logical content identity and does not establish retention.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CatalogDigest([u8; 32]);

impl CatalogDigest {
    /// Returns the exact canonical digest bytes.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub(crate) const fn from_validated(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}
