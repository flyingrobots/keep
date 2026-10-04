//! This module owns exact migration catalog-to-segment record binding.

use crate::adapters::{
    AdmittedSegment, AdmittedSegmentRecord, CatalogAdmissionError, DecodedCatalogEntry,
};

use super::migration_catalog_plan::PlannedEntry;

pub(super) fn validate(
    entries: &[PlannedEntry],
    segment: &AdmittedSegment<'_>,
) -> Result<(), CatalogAdmissionError> {
    let digest = segment.digest();
    let mut pending = entries.iter().peekable();
    let mut cursor = segment.record_cursor();
    while let Some(located) =
        cursor
            .next_record()
            .map_err(|source| CatalogAdmissionError::Segment {
                digest,
                source: Box::new(source),
            })?
    {
        refuse_preceding(&mut pending, located.offset)?;
        validate_at_offset(&mut pending, located.offset, located.record)?;
    }
    cursor
        .finish()
        .map_err(|source| CatalogAdmissionError::Segment {
            digest,
            source: Box::new(source),
        })?;
    pending
        .next()
        .map_or(Ok(()), |entry| Err(location_error(entry.entry)))
}

fn refuse_preceding(
    pending: &mut std::iter::Peekable<std::slice::Iter<'_, PlannedEntry>>,
    record_offset: u64,
) -> Result<(), CatalogAdmissionError> {
    match pending.peek() {
        Some(entry) if entry.entry.record_offset() < record_offset => {
            Err(location_error(entry.entry))
        }
        Some(_) | None => Ok(()),
    }
}

fn validate_at_offset(
    pending: &mut std::iter::Peekable<std::slice::Iter<'_, PlannedEntry>>,
    record_offset: u64,
    record: AdmittedSegmentRecord<'_>,
) -> Result<(), CatalogAdmissionError> {
    while matches!(pending.peek(), Some(entry) if entry.entry.record_offset() == record_offset) {
        let Some(entry) = pending.next() else {
            break;
        };
        validate_record(entry.entry, record)?;
    }
    Ok(())
}

fn validate_record(
    entry: DecodedCatalogEntry,
    record: AdmittedSegmentRecord<'_>,
) -> Result<(), CatalogAdmissionError> {
    if record.header().record_length() != entry.record_length() {
        return Err(location_error(entry));
    }
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

const fn location_error(entry: DecodedCatalogEntry) -> CatalogAdmissionError {
    CatalogAdmissionError::LocationNotTopLevel {
        identity: entry.identity(),
        segment_digest: entry.segment_digest(),
        record_offset: entry.record_offset(),
        record_length: entry.record_length().get(),
    }
}
