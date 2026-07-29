//! Owned canonical catalog bytes derived from admitted segments.

use super::checksummed_catalog::CatalogMetadata;
use super::{AdmittedSegment, CatalogEncodeError, ChecksummedCatalog, catalog_encoder};
use crate::{CatalogDigest, CatalogGeneration};

/// Owned canonical version-1 catalog bytes.
///
/// Construction derives physical record coordinates from fully admitted
/// segments, sorts entries by logical identity, and refuses duplicate records
/// or an unreferenced segment. The complete catalog is materialized in memory
/// with the version-1 entry-count and byte-length bounds enforced before
/// allocation.
#[must_use]
#[derive(Debug, Eq, PartialEq)]
pub struct CanonicalCatalog {
    encoded: Vec<u8>,
    metadata: CatalogMetadata,
    digest: CatalogDigest,
}

impl CanonicalCatalog {
    /// Derives one complete canonical catalog from all records in `segments`.
    ///
    /// # Errors
    ///
    /// Returns [`CatalogEncodeError`] for an invalid predecessor law, checked
    /// count or length refusal, unreferenced segment, allocation failure,
    /// failed immutable segment revalidation, or duplicate logical identity.
    pub fn from_segments(
        generation: CatalogGeneration,
        previous_catalog_digest: Option<CatalogDigest>,
        segments: &[AdmittedSegment<'_>],
    ) -> Result<Self, CatalogEncodeError> {
        catalog_encoder::encode(generation, previous_catalog_digest, segments)
    }

    /// Returns the complete exact canonical bytes.
    #[must_use]
    pub fn encoded(&self) -> &[u8] {
        &self.encoded
    }

    /// Borrows the generated catalog with its construction-time integrity proof.
    pub fn checksummed(&self) -> ChecksummedCatalog<'_> {
        ChecksummedCatalog::from_verified_parts(&self.encoded, self.metadata, self.digest)
    }

    pub(super) const fn admitted(
        encoded: Vec<u8>,
        metadata: CatalogMetadata,
        digest: CatalogDigest,
    ) -> Self {
        Self {
            encoded,
            metadata,
            digest,
        }
    }
}
