//! This module owns the registered version-2 format-definition digest.

/// Identity of one registered store-format definition.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StoreFormatDefinitionDigest([u8; 32]);

impl StoreFormatDefinitionDigest {
    /// Digest of the frozen `keep.segment-store/v2` definition.
    pub const VERSION_TWO: Self = Self([
        0x6c, 0xbc, 0x1c, 0x75, 0xf6, 0xef, 0xab, 0x18, 0xc7, 0xc5, 0x0a, 0xe2, 0x81, 0xed, 0xef,
        0x77, 0xa8, 0xb0, 0xc9, 0xba, 0x19, 0xf6, 0x18, 0xb2, 0x8b, 0x18, 0x48, 0x3d, 0x08, 0x46,
        0x2c, 0x92,
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
