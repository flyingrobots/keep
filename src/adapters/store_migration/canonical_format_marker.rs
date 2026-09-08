//! This boundary module owns canonical version-2 store-format marker bytes.

use super::{StoreFormatMarkerDigest, format_marker_decoder, format_marker_encoder};

/// Owned canonical version-2 store-format marker.
#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalStoreFormatMarker {
    encoded: [u8; format_marker_decoder::ENCODED_LENGTH],
    digest: StoreFormatMarkerDigest,
}

impl CanonicalStoreFormatMarker {
    /// Constructs the one registered version-2 marker.
    pub fn version_two() -> Self {
        format_marker_encoder::version_two()
    }

    /// Returns the canonical bytes.
    #[must_use]
    pub const fn encoded(&self) -> &[u8] {
        &self.encoded
    }

    /// Returns the identity of all marker bytes.
    pub const fn digest(&self) -> StoreFormatMarkerDigest {
        self.digest
    }

    pub(super) const fn admitted(
        encoded: [u8; format_marker_decoder::ENCODED_LENGTH],
        digest: StoreFormatMarkerDigest,
    ) -> Self {
        Self { encoded, digest }
    }
}
