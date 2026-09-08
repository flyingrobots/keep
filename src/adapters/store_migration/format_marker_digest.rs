//! This module owns version-2 store-format marker identity.

/// Domain-separated identity of all canonical store-format marker bytes.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StoreFormatMarkerDigest([u8; 32]);

impl StoreFormatMarkerDigest {
    /// Returns the raw digest bytes.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub(super) const fn from_hash(hash: [u8; 32]) -> Self {
        Self(hash)
    }
}
