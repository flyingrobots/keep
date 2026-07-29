//! Framing- and checksum-verified borrowed publication head.

use super::{
    AdmittedCatalog, CatalogSnapshot, CatalogSnapshotError, PublicationHeadDecodeError,
    catalog_snapshot_admission, publication_head_decoder,
};
use crate::{CatalogDigest, CatalogGeneration, CatalogLength};

/// Borrowed publication-head bytes with canonical framing and checksum proof.
///
/// This state does not prove that the named catalog exists or that any catalog
/// entry names an admitted segment record. A reader must not treat it as a
/// complete catalog snapshot.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ChecksummedPublicationHead<'a> {
    encoded: &'a [u8],
    generation: CatalogGeneration,
    catalog_length: CatalogLength,
    catalog_digest: CatalogDigest,
}

impl<'a> ChecksummedPublicationHead<'a> {
    /// Decodes exact version-1 framing and verifies the head checksum.
    ///
    /// This operation performs no allocation or I/O.
    ///
    /// # Errors
    ///
    /// Returns [`PublicationHeadDecodeError`] for wrong framing, unsupported or
    /// noncanonical fields, invalid coordinates, or checksum disagreement.
    pub fn decode(encoded: &'a [u8]) -> Result<Self, PublicationHeadDecodeError> {
        publication_head_decoder::decode(encoded)
    }

    /// Pins this head to one fully admitted catalog generation.
    ///
    /// # Errors
    ///
    /// Returns [`CatalogSnapshotError`] when the generation, catalog length, or
    /// physical catalog digest differs.
    pub fn admit<'catalog, 'records>(
        self,
        catalog: AdmittedCatalog<'catalog, 'records>,
    ) -> Result<CatalogSnapshot<'a, 'catalog, 'records>, CatalogSnapshotError> {
        catalog_snapshot_admission::admit(self, catalog)
    }

    /// Returns the exact borrowed canonical bytes.
    #[must_use]
    pub const fn encoded(self) -> &'a [u8] {
        self.encoded
    }

    /// Returns the positive generation named by the head.
    pub const fn generation(self) -> CatalogGeneration {
        self.generation
    }

    /// Returns the canonical length of the named catalog.
    pub const fn catalog_length(self) -> CatalogLength {
        self.catalog_length
    }

    /// Returns the physical digest of the named catalog.
    pub const fn catalog_digest(self) -> CatalogDigest {
        self.catalog_digest
    }

    pub(super) const fn from_verified_parts(
        encoded: &'a [u8],
        generation: CatalogGeneration,
        catalog_length: CatalogLength,
        catalog_digest: CatalogDigest,
    ) -> Self {
        Self {
            encoded,
            generation,
            catalog_length,
            catalog_digest,
        }
    }
}
