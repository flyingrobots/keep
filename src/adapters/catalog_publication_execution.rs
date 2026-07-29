//! Ordered execution of a fully preflighted catalog publication.

use std::io;

use super::{
    CanonicalCatalog, CanonicalPublicationHead, CatalogPublicationError,
    CatalogPublicationExpectation, CatalogPublicationPhase, CatalogPublicationReadiness,
    CatalogPublicationStorage, CatalogSnapshot, ChecksummedCatalog,
};

pub(super) fn execute_current(
    storage: &mut impl CatalogPublicationStorage,
    expectation: CatalogPublicationExpectation,
    candidate: &CatalogSnapshot<'_, '_, '_>,
) -> Result<CatalogPublicationReadiness, CatalogPublicationError> {
    let readiness = phase(
        CatalogPublicationPhase::VerifyCurrent,
        storage.verify_current(expectation, candidate),
    )?;
    if readiness == CatalogPublicationReadiness::AlreadyPublished {
        phase(
            CatalogPublicationPhase::SynchronizeRoot,
            storage.synchronize_root(),
        )?;
    }
    Ok(readiness)
}

pub(super) fn execute_segment(
    storage: &mut impl CatalogPublicationStorage,
    segment: &super::AdmittedSegment<'_>,
) -> Result<(), CatalogPublicationError> {
    phase(
        CatalogPublicationPhase::LinkSegment,
        storage.link_segment(segment),
    )?;
    phase(
        CatalogPublicationPhase::VerifySegmentPool,
        storage.verify_segment_pool(segment),
    )?;
    phase(
        CatalogPublicationPhase::SynchronizeSegments,
        storage.synchronize_segments(),
    )?;
    phase(
        CatalogPublicationPhase::RemoveSegmentStage,
        storage.remove_segment_stage(),
    )?;
    phase(
        CatalogPublicationPhase::SynchronizeStagingAfterSegment,
        storage.synchronize_staging_after_segment(),
    )
}

pub(super) fn execute_catalog(
    storage: &mut impl CatalogPublicationStorage,
    catalog: &CanonicalCatalog,
    checksummed: ChecksummedCatalog<'_>,
) -> Result<(), CatalogPublicationError> {
    phase(
        CatalogPublicationPhase::CreateCatalogStage,
        storage.create_catalog_stage(),
    )?;
    phase(
        CatalogPublicationPhase::WriteCatalog,
        storage.write_catalog(catalog),
    )?;
    phase(
        CatalogPublicationPhase::FlushCatalog,
        storage.flush_catalog(),
    )?;
    phase(
        CatalogPublicationPhase::SynchronizeCatalog,
        storage.synchronize_catalog(),
    )?;
    phase(
        CatalogPublicationPhase::LinkCatalog,
        storage.link_catalog(checksummed),
    )?;
    phase(
        CatalogPublicationPhase::VerifyCatalogPool,
        storage.verify_catalog_pool(checksummed),
    )?;
    phase(
        CatalogPublicationPhase::SynchronizeCatalogs,
        storage.synchronize_catalogs(),
    )?;
    phase(
        CatalogPublicationPhase::RemoveCatalogStage,
        storage.remove_catalog_stage(),
    )?;
    phase(
        CatalogPublicationPhase::SynchronizeStagingAfterCatalog,
        storage.synchronize_staging_after_catalog(),
    )
}

pub(super) fn execute_head(
    storage: &mut impl CatalogPublicationStorage,
    head: &CanonicalPublicationHead,
    snapshot: &CatalogSnapshot<'_, '_, '_>,
) -> Result<(), CatalogPublicationError> {
    phase(
        CatalogPublicationPhase::CreateHeadStage,
        storage.create_head_stage(),
    )?;
    phase(CatalogPublicationPhase::WriteHead, storage.write_head(head))?;
    phase(CatalogPublicationPhase::FlushHead, storage.flush_head())?;
    phase(
        CatalogPublicationPhase::SynchronizeHead,
        storage.synchronize_head(),
    )?;
    phase(
        CatalogPublicationPhase::VerifyHeadView,
        storage.verify_head_view(head, snapshot),
    )?;
    phase(CatalogPublicationPhase::ReplaceHead, storage.replace_head())?;
    phase(
        CatalogPublicationPhase::SynchronizeRoot,
        storage.synchronize_root(),
    )
}

fn phase<T>(
    phase: CatalogPublicationPhase,
    result: io::Result<T>,
) -> Result<T, CatalogPublicationError> {
    result.map_err(|source| CatalogPublicationError::storage(phase, source))
}
