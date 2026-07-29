//! Binding of catalog-local coordinates to exact admitted segment records.

use super::catalog_entry_plan::{self, CatalogEntryPlan};
use super::{
    AdmittedCatalog, AdmittedSegment, AdmittedSegmentRecord, CatalogAdmissionError,
    CatalogAllocationPhase, CatalogRecordBinding, ChecksummedCatalog, DecodedCatalogEntry,
    SegmentDigest,
};

pub(super) fn admit<'catalog, 'records>(
    catalog: ChecksummedCatalog<'catalog>,
    segments: &[AdmittedSegment<'records>],
) -> Result<AdmittedCatalog<'catalog, 'records>, CatalogAdmissionError> {
    let requested = usize::try_from(catalog.entry_count()).map_err(|_source| {
        CatalogAdmissionError::EntryCountHostWidth {
            observed: catalog.entry_count(),
        }
    })?;
    if segments.len() > requested {
        return Err(CatalogAdmissionError::SegmentCountOutOfBounds {
            maximum: catalog.entry_count(),
            observed: segments.len(),
        });
    }
    let segment_index = index_segments(segments)?;
    let mut plan = plan_entries(catalog, &segment_index, requested)?;
    catalog_entry_plan::bind(&mut plan, &segment_index)?;
    plan.sort_unstable_by_key(CatalogEntryPlan::ordinal);
    let mut bindings = Vec::new();
    bindings
        .try_reserve_exact(requested)
        .map_err(|source| CatalogAdmissionError::Allocation {
            phase: CatalogAllocationPhase::RecordBindings,
            requested,
            source,
        })?;
    for planned in plan {
        let (entry, record) = planned.into_bound()?;
        validate_record(entry, record)?;
        bindings.push(CatalogRecordBinding::new(entry.identity(), record));
    }
    Ok(AdmittedCatalog::from_verified_parts(catalog, bindings))
}

fn plan_entries<'segments, 'records>(
    catalog: ChecksummedCatalog<'_>,
    segments: &[&'segments AdmittedSegment<'records>],
    requested: usize,
) -> Result<Vec<CatalogEntryPlan<'segments, 'records>>, CatalogAdmissionError> {
    let mut plan = Vec::new();
    plan.try_reserve_exact(requested)
        .map_err(|source| CatalogAdmissionError::Allocation {
            phase: CatalogAllocationPhase::EntryPlan,
            requested,
            source,
        })?;
    let entries = catalog
        .entries()
        .map_err(|source| CatalogAdmissionError::Catalog { source })?;
    for (ordinal, entry) in entries.enumerate() {
        let entry = entry.map_err(|source| CatalogAdmissionError::Catalog { source })?;
        let segment = find_segment(segments, entry.segment_digest())?;
        plan.push(CatalogEntryPlan::new(ordinal, entry, segment));
    }
    Ok(plan)
}

fn index_segments<'slice, 'records>(
    segments: &'slice [AdmittedSegment<'records>],
) -> Result<Vec<&'slice AdmittedSegment<'records>>, CatalogAdmissionError> {
    let requested = segments.len();
    let mut indexed = Vec::new();
    indexed
        .try_reserve_exact(requested)
        .map_err(|source| CatalogAdmissionError::Allocation {
            phase: CatalogAllocationPhase::SegmentIndex,
            requested,
            source,
        })?;
    indexed.extend(segments);
    indexed.sort_unstable_by_key(|segment| segment.digest());
    for pair in indexed.windows(2) {
        let [first, second] = pair else {
            continue;
        };
        if first.digest() == second.digest() {
            return Err(CatalogAdmissionError::DuplicateSegment {
                digest: first.digest(),
            });
        }
    }
    Ok(indexed)
}

fn find_segment<'segments, 'records>(
    segments: &[&'segments AdmittedSegment<'records>],
    digest: SegmentDigest,
) -> Result<&'segments AdmittedSegment<'records>, CatalogAdmissionError> {
    let index = segments
        .binary_search_by_key(&digest, |segment| segment.digest())
        .map_err(|_source| CatalogAdmissionError::MissingSegment { digest })?;
    segments
        .get(index)
        .copied()
        .ok_or(CatalogAdmissionError::MissingSegment { digest })
}

fn validate_record(
    entry: DecodedCatalogEntry,
    record: AdmittedSegmentRecord<'_>,
) -> Result<(), CatalogAdmissionError> {
    if record.identity() != entry.identity() {
        return Err(CatalogAdmissionError::RecordIdentityMismatch {
            expected: entry.identity(),
            observed: record.identity(),
        });
    }
    if record.checksum() != entry.checksum() {
        return Err(CatalogAdmissionError::RecordChecksumMismatch {
            expected: entry.checksum(),
            observed: record.checksum(),
        });
    }
    Ok(())
}
