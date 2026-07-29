//! Binding of catalog-local coordinates to exact admitted segment records.

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
    let mut bindings = Vec::new();
    bindings
        .try_reserve_exact(requested)
        .map_err(|source| CatalogAdmissionError::Allocation {
            phase: CatalogAllocationPhase::RecordBindings,
            requested,
            source,
        })?;
    let entries = catalog
        .entries()
        .map_err(|source| CatalogAdmissionError::Catalog { source })?;
    for entry in entries {
        let entry = entry.map_err(|source| CatalogAdmissionError::Catalog { source })?;
        let segment = find_segment(&segment_index, entry.segment_digest())?;
        let record = locate_record(segment, entry)?;
        validate_record(entry, record)?;
        bindings.push(CatalogRecordBinding::new(entry.identity(), record));
    }
    Ok(AdmittedCatalog::from_verified_parts(catalog, bindings))
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

fn find_segment<'slice, 'records>(
    segments: &'slice [&AdmittedSegment<'records>],
    digest: SegmentDigest,
) -> Result<&'slice AdmittedSegment<'records>, CatalogAdmissionError> {
    let index = segments
        .binary_search_by_key(&digest, |segment| segment.digest())
        .map_err(|_source| CatalogAdmissionError::MissingSegment { digest })?;
    segments
        .get(index)
        .copied()
        .ok_or(CatalogAdmissionError::MissingSegment { digest })
}

fn locate_record<'records>(
    segment: &AdmittedSegment<'records>,
    entry: DecodedCatalogEntry,
) -> Result<AdmittedSegmentRecord<'records>, CatalogAdmissionError> {
    let digest = segment.digest();
    let mut cursor = segment.record_cursor();
    let mut found = None;
    while let Some(located) =
        cursor
            .next_record()
            .map_err(|source| CatalogAdmissionError::Segment {
                digest,
                source: Box::new(source),
            })?
    {
        if located.offset == entry.record_offset()
            && located.record.header().record_length() == entry.record_length()
        {
            found = Some(located.record);
        }
    }
    cursor
        .finish()
        .map_err(|source| CatalogAdmissionError::Segment {
            digest,
            source: Box::new(source),
        })?;
    found.ok_or_else(|| CatalogAdmissionError::LocationNotTopLevel {
        identity: entry.identity(),
        segment_digest: entry.segment_digest(),
        record_offset: entry.record_offset(),
        record_length: entry.record_length().get(),
    })
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
