//! Catalog whose logical records are bound to admitted segment bytes.

use super::{
    AdmittedSegmentRecord, CatalogRecordBinding, CatalogSuccessor, CatalogTransitionError,
    ChecksummedCatalog, SegmentRecordIdentity, catalog_transition,
};
use crate::{CatalogDigest, CatalogGeneration, CatalogLength};

/// Immutable admitted catalog over exact content-admitted segment records.
///
/// Lookups expose logical identities and verified record bytes. Physical
/// segment names, offsets, and lengths remain representation details.
#[must_use]
#[derive(Debug)]
pub struct AdmittedCatalog<'catalog, 'records> {
    catalog: ChecksummedCatalog<'catalog>,
    records: Vec<CatalogRecordBinding<'records>>,
}

impl<'catalog, 'records> AdmittedCatalog<'catalog, 'records> {
    /// Returns the immutable catalog generation.
    pub const fn generation(&self) -> CatalogGeneration {
        self.catalog.generation()
    }

    /// Returns the verified physical catalog digest.
    pub const fn digest(&self) -> CatalogDigest {
        self.catalog.digest()
    }

    /// Returns the predecessor witness, absent only for generation 1.
    #[must_use]
    pub const fn previous_catalog_digest(&self) -> Option<CatalogDigest> {
        self.catalog.previous_catalog_digest()
    }

    pub(crate) const fn length(&self) -> CatalogLength {
        self.catalog.length()
    }

    /// Returns the exact number of logical record bindings.
    #[must_use]
    pub const fn record_count(&self) -> u64 {
        self.catalog.entry_count()
    }

    /// Looks up one logical identity without exposing its physical location.
    #[must_use]
    pub fn record(
        &self,
        identity: SegmentRecordIdentity,
    ) -> Option<AdmittedSegmentRecord<'records>> {
        let index = self
            .records
            .binary_search_by_key(&identity, |binding| binding.identity())
            .ok()?;
        self.records
            .get(index)
            .copied()
            .map(CatalogRecordBinding::record)
    }

    /// Admits a fully verified candidate as this snapshot's exact successor.
    ///
    /// # Errors
    ///
    /// Returns [`CatalogTransitionError`] when generation arithmetic is
    /// exhausted, the candidate is not exactly one generation later, or its
    /// predecessor digest does not equal this catalog's verified digest.
    pub fn validate_successor<'next_catalog, 'next_records>(
        &self,
        candidate: AdmittedCatalog<'next_catalog, 'next_records>,
    ) -> Result<CatalogSuccessor<'next_catalog, 'next_records>, CatalogTransitionError> {
        catalog_transition::validate(self, candidate)
    }

    pub(super) const fn from_verified_parts(
        catalog: ChecksummedCatalog<'catalog>,
        records: Vec<CatalogRecordBinding<'records>>,
    ) -> Self {
        Self { catalog, records }
    }
}
