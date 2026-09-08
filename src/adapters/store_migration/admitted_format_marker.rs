//! This boundary module owns admitted version-2 store-format markers.

use super::{
    StoreFormatDefinitionDigest, StoreFormatMarkerDecodeError, StoreFormatMarkerDigest,
    format_marker_decoder,
};

/// Borrowed canonical marker bytes with verified version-2 format identity.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AdmittedStoreFormatMarker<'encoded> {
    encoded: &'encoded [u8],
    definition_digest: StoreFormatDefinitionDigest,
    digest: StoreFormatMarkerDigest,
}

impl<'encoded> AdmittedStoreFormatMarker<'encoded> {
    /// Decodes and verifies one exact version-2 store-format marker.
    ///
    /// This operation performs no allocation or I/O.
    ///
    /// # Errors
    ///
    /// Returns [`StoreFormatMarkerDecodeError`] for wrong framing,
    /// unsupported fields, checksum disagreement, or an unregistered
    /// definition or namespace bound.
    pub fn decode(encoded: &'encoded [u8]) -> Result<Self, StoreFormatMarkerDecodeError> {
        format_marker_decoder::decode(encoded)
    }

    /// Returns the exact borrowed canonical bytes.
    #[must_use]
    pub const fn encoded(&self) -> &'encoded [u8] {
        self.encoded
    }

    /// Returns the registered format-definition digest.
    pub const fn definition_digest(&self) -> StoreFormatDefinitionDigest {
        self.definition_digest
    }

    /// Returns the identity of all marker bytes.
    pub const fn digest(&self) -> StoreFormatMarkerDigest {
        self.digest
    }

    pub(super) const fn admitted(
        encoded: &'encoded [u8],
        definition_digest: StoreFormatDefinitionDigest,
        digest: StoreFormatMarkerDigest,
    ) -> Self {
        Self {
            encoded,
            definition_digest,
            digest,
        }
    }
}
