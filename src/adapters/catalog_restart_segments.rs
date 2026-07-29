//! This module owns bounded loading of catalog-selected immutable segments.

use cap_std::fs::Dir;

use super::loaded_segment::LoadedSegment;
use super::segment_header::MAXIMUM_SEGMENT_LENGTH;
use super::{
    AdmittedSegment, CatalogRestartArtifact, CatalogRestartError, CatalogRestartPhase,
    CatalogRestartPolicy, ChecksummedCatalog, SegmentDigest, catalog_restart_io,
    physical_pool_name,
};

pub(super) fn collect(
    catalog: ChecksummedCatalog<'_>,
) -> Result<Vec<SegmentDigest>, CatalogRestartError> {
    let capacity = usize::try_from(catalog.entry_count()).map_err(|_source| {
        CatalogRestartError::Allocation {
            artifact: CatalogRestartArtifact::Catalog,
            byte_count: catalog.entry_count(),
            source: None,
        }
    })?;
    let mut digests = Vec::new();
    digests
        .try_reserve_exact(capacity)
        .map_err(|source| CatalogRestartError::Allocation {
            artifact: CatalogRestartArtifact::Catalog,
            byte_count: catalog.entry_count(),
            source: Some(source),
        })?;
    for entry in catalog
        .entries()
        .map_err(|source| CatalogRestartError::Catalog { source })?
    {
        let entry = entry.map_err(|source| CatalogRestartError::Catalog { source })?;
        digests.push(entry.segment_digest());
    }
    digests.sort_unstable();
    digests.dedup();
    Ok(digests)
}

pub(super) fn load(
    directory: &Dir,
    digests: &[SegmentDigest],
    policy: CatalogRestartPolicy,
) -> Result<Vec<LoadedSegment>, CatalogRestartError> {
    let segment_count =
        u64::try_from(digests.len()).map_err(|_source| CatalogRestartError::SegmentIndexLength)?;
    let mut loaded = Vec::new();
    loaded.try_reserve_exact(digests.len()).map_err(|source| {
        CatalogRestartError::SegmentIndexAllocation {
            segment_count,
            source,
        }
    })?;
    let mut retained = 0_u64;
    for digest in digests {
        let artifact = CatalogRestartArtifact::Segment { digest: *digest };
        let name = physical_pool_name::segment(*digest);
        let (file, observed) = catalog_restart_io::open_regular(
            directory,
            &name,
            artifact,
            CatalogRestartPhase::OpenSegment,
        )?;
        if observed > MAXIMUM_SEGMENT_LENGTH {
            return Err(CatalogRestartError::Length {
                artifact,
                minimum: 0,
                maximum: MAXIMUM_SEGMENT_LENGTH,
                observed,
            });
        }
        retained = retained.checked_add(observed).ok_or(
            CatalogRestartError::RetainedSegmentByteArithmetic {
                current: retained,
                addition: observed,
            },
        )?;
        if retained > policy.retained_segment_bytes().get() {
            return Err(CatalogRestartError::RetainedSegmentBytes {
                maximum: policy.retained_segment_bytes().get(),
                observed: retained,
            });
        }
        let encoded = catalog_restart_io::read_exact(
            file,
            artifact,
            CatalogRestartPhase::ReadSegment,
            observed,
        )?;
        let segment =
            AdmittedSegment::decode(&encoded, policy.segment_read()).map_err(|source| {
                CatalogRestartError::Segment {
                    expected: *digest,
                    source: Box::new(source),
                }
            })?;
        if segment.digest() != *digest {
            return Err(CatalogRestartError::SegmentCoordinate {
                expected: *digest,
                observed: segment.digest(),
            });
        }
        loaded.push(LoadedSegment::new(*digest, encoded));
    }
    Ok(loaded)
}
