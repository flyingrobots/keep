//! Owned canonical publication-head bytes.

use super::{ChecksummedCatalog, publication_head_encoder};

/// Owned canonical version-1 publication head.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CanonicalPublicationHead {
    encoded: [u8; 128],
}

impl CanonicalPublicationHead {
    /// Emits the exact head for one checksum- and digest-verified catalog.
    pub fn for_catalog(catalog: ChecksummedCatalog<'_>) -> Self {
        publication_head_encoder::encode(catalog)
    }

    /// Returns the complete exact canonical bytes.
    #[must_use]
    pub const fn encoded(&self) -> &[u8; 128] {
        &self.encoded
    }

    pub(super) const fn admitted(encoded: [u8; 128]) -> Self {
        Self { encoded }
    }
}
