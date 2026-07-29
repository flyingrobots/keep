//! Catalog whose logical records are bound to admitted segment bytes.

use super::{
    AdmittedSegmentRecord, CatalogRecordBinding, ChecksummedCatalog, SegmentRecordIdentity,
};
use crate::{CatalogDigest, CatalogGeneration};

/// Immutable catalog snapshot over exact content-admitted segment records.
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

    pub(super) const fn from_verified_parts(
        catalog: ChecksummedCatalog<'catalog>,
        records: Vec<CatalogRecordBinding<'records>>,
    ) -> Self {
        Self { catalog, records }
    }
}
