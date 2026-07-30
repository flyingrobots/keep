//! Immutable reader snapshot pinned by one head and admitted catalog.

use super::{
    AdmittedCatalog, AdmittedSegmentRecord, ChecksummedPublicationHead, SegmentRecordIdentity,
};
use crate::{CatalogDigest, CatalogGeneration, CatalogLength};

/// One complete immutable catalog generation pinned by a checksummed head.
///
/// The snapshot owns both proofs. Later reads of the mutable head cannot change
/// its generation, logical bindings, or borrowed record bytes.
#[must_use]
#[derive(Debug)]
pub struct CatalogSnapshot<'head, 'catalog, 'records> {
    head: ChecksummedPublicationHead<'head>,
    catalog: AdmittedCatalog<'catalog, 'records>,
}

impl<'head, 'catalog, 'records> CatalogSnapshot<'head, 'catalog, 'records> {
    /// Returns the exact pinned generation.
    pub const fn generation(&self) -> CatalogGeneration {
        self.head.generation()
    }

    /// Returns the verified physical digest pinned by the head.
    pub const fn catalog_digest(&self) -> CatalogDigest {
        self.head.catalog_digest()
    }

    /// Returns the verified catalog byte length pinned by the head.
    pub const fn catalog_length(&self) -> CatalogLength {
        self.catalog.length()
    }

    /// Returns the verified predecessor coordinate from the admitted catalog.
    pub const fn previous_catalog_digest(&self) -> Option<CatalogDigest> {
        self.catalog.previous_catalog_digest()
    }

    /// Returns the exact number of logical record bindings.
    #[must_use]
    pub const fn record_count(&self) -> u64 {
        self.catalog.record_count()
    }

    /// Looks up one logical record within this pinned generation.
    #[must_use]
    pub fn record(
        &self,
        identity: SegmentRecordIdentity,
    ) -> Option<AdmittedSegmentRecord<'records>> {
        self.catalog.record(identity)
    }

    pub(super) const fn new(
        head: ChecksummedPublicationHead<'head>,
        catalog: AdmittedCatalog<'catalog, 'records>,
    ) -> Self {
        Self { head, catalog }
    }
}
