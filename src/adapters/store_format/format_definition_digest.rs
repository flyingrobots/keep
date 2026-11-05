//! This module owns the registered version-2 format-definition digest.

/// Identity of one registered store-format definition.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StoreFormatDefinitionDigest([u8; 32]);

impl StoreFormatDefinitionDigest {
    /// Digest of the frozen `keep.segment-store/v2` definition.
    pub const VERSION_TWO: Self = Self([
        0xf1, 0xa0, 0xce, 0x10, 0xc0, 0xd3, 0x63, 0xd1, 0x45, 0xf8, 0xa6, 0x93, 0xea, 0x22, 0x65,
        0x1e, 0x47, 0xcf, 0x35, 0x6c, 0xaa, 0x14, 0xf1, 0xef, 0x4f, 0x89, 0x58, 0xd8, 0xb2, 0x9f,
        0x36, 0x1d,
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
