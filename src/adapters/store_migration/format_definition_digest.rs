//! This module owns the registered version-2 format-definition digest.

/// Identity of one registered store-format definition.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StoreFormatDefinitionDigest([u8; 32]);

impl StoreFormatDefinitionDigest {
    /// Digest of the frozen `keep.segment-store/v2` definition.
    pub const VERSION_TWO: Self = Self([
        0x32, 0x38, 0x1f, 0x1a, 0xc3, 0x32, 0xd1, 0x27, 0x7a, 0x7e, 0x1f, 0xaf, 0x8f, 0x11, 0x57,
        0x69, 0x93, 0xcb, 0x55, 0xb7, 0xe8, 0x5d, 0x2a, 0x11, 0x0b, 0x74, 0xdc, 0x9c, 0x3b, 0x87,
        0x34, 0x27,
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
