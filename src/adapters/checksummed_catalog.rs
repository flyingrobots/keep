//! Canonically framed, checksum- and digest-verified borrowed catalog.

use super::{CatalogDecodeError, catalog_decoder};
use crate::{CatalogDigest, CatalogGeneration};

/// Borrowed catalog bytes with canonical framing, ordering, and integrity proof.
///
/// This state validates catalog-local fields and entry coordinates. It does not
/// prove that any named segment exists or that a location selects a top-level
/// admitted record. Callers must not treat it as a reader snapshot.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ChecksummedCatalog<'a> {
    encoded: &'a [u8],
    generation: CatalogGeneration,
    previous_catalog_digest: Option<CatalogDigest>,
    entry_count: u64,
    digest: CatalogDigest,
}

impl<'a> ChecksummedCatalog<'a> {
    /// Decodes one exact version-1 catalog and verifies local integrity.
    ///
    /// Admission streams fixed-width entries without heap allocation or I/O.
    ///
    /// # Errors
    ///
    /// Returns [`CatalogDecodeError`] for malformed, unsupported,
    /// noncanonical, unordered, duplicate, corrupt, or over-limit bytes.
    pub fn decode(encoded: &'a [u8]) -> Result<Self, CatalogDecodeError> {
        catalog_decoder::decode(encoded)
    }

    /// Returns the exact borrowed canonical bytes.
    #[must_use]
    pub const fn encoded(self) -> &'a [u8] {
        self.encoded
    }

    /// Returns the positive catalog generation.
    pub const fn generation(self) -> CatalogGeneration {
        self.generation
    }

    /// Returns the predecessor witness, absent only for generation 1.
    #[must_use]
    pub const fn previous_catalog_digest(self) -> Option<CatalogDigest> {
        self.previous_catalog_digest
    }

    /// Returns the exact bounded entry count.
    #[must_use]
    pub const fn entry_count(self) -> u64 {
        self.entry_count
    }

    /// Returns the verified physical catalog digest.
    pub const fn digest(self) -> CatalogDigest {
        self.digest
    }

    pub(super) const fn from_verified_parts(
        encoded: &'a [u8],
        generation: CatalogGeneration,
        previous_catalog_digest: Option<CatalogDigest>,
        entry_count: u64,
        digest: CatalogDigest,
    ) -> Self {
        Self {
            encoded,
            generation,
            previous_catalog_digest,
            entry_count,
            digest,
        }
    }
}
