//! This module owns the registered version-2 format-definition digest.

/// Identity of one registered store-format definition.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StoreFormatDefinitionDigest([u8; 32]);

impl StoreFormatDefinitionDigest {
    /// Digest of the frozen `keep.segment-store/v2` definition.
    pub const VERSION_TWO: Self = Self([
        0xa4, 0xa0, 0x10, 0xce, 0xe5, 0xda, 0x8a, 0xa3, 0xba, 0x15, 0x3c, 0x50, 0x34, 0xf4, 0x36,
        0xb9, 0x27, 0x42, 0xc6, 0xc1, 0xf7, 0xcf, 0x6b, 0x43, 0xd8, 0x90, 0xad, 0x5f, 0xd5, 0xb5,
        0xcf, 0x89,
    ]);

    /// Returns the raw digest bytes.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub(super) const fn from_hash(hash: [u8; 32]) -> Self {
        Self(hash)
    }
}
