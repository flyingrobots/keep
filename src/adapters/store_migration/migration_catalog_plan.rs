//! This module owns bounded physical catalog-entry planning for migration.

use crate::adapters::{
    CatalogAdmissionError, CatalogAllocationPhase, ChecksummedCatalog, DecodedCatalogEntry,
    SegmentDigest,
};

use super::migration_catalog_admission::{MigrationCatalogAdmissionError, catalog_error};

#[derive(Clone, Copy)]
pub(super) struct PlannedEntry {
    pub(super) ordinal: usize,
    pub(super) entry: DecodedCatalogEntry,
}

impl PlannedEntry {
    pub(super) const fn physical_order(self) -> (SegmentDigest, u64, usize) {
        (
            self.entry.segment_digest(),
            self.entry.record_offset(),
            self.ordinal,
        )
    }
}

pub(super) fn plan<E>(
    catalog: ChecksummedCatalog<'_>,
) -> Result<Vec<PlannedEntry>, MigrationCatalogAdmissionError<E>> {
    let requested = usize::try_from(catalog.entry_count()).map_err(|_source| {
        catalog_error(CatalogAdmissionError::EntryCountHostWidth {
            observed: catalog.entry_count(),
        })
    })?;
    let mut plan = Vec::new();
    plan.try_reserve_exact(requested).map_err(|source| {
        catalog_error(CatalogAdmissionError::Allocation {
            phase: CatalogAllocationPhase::EntryPlan,
            requested,
            source,
        })
    })?;
    let entries = catalog
        .entries()
        .map_err(|source| catalog_error(CatalogAdmissionError::Catalog { source }))?;
    for (ordinal, entry) in entries.enumerate() {
        let entry =
            entry.map_err(|source| catalog_error(CatalogAdmissionError::Catalog { source }))?;
        plan.push(PlannedEntry { ordinal, entry });
    }
    Ok(plan)
}
