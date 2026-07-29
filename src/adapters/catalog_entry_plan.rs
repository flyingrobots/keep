//! This module owns one-pass physical record lookup for catalog admission.

use super::{
    AdmittedSegment, AdmittedSegmentRecord, CatalogAdmissionError, DecodedCatalogEntry,
    SegmentDigest,
};

pub(super) struct CatalogEntryPlan<'segments, 'records> {
    ordinal: usize,
    entry: DecodedCatalogEntry,
    segment: &'segments AdmittedSegment<'records>,
    record: Option<AdmittedSegmentRecord<'records>>,
}

impl<'segments, 'records> CatalogEntryPlan<'segments, 'records> {
    pub(super) const fn new(
        ordinal: usize,
        entry: DecodedCatalogEntry,
        segment: &'segments AdmittedSegment<'records>,
    ) -> Self {
        Self {
            ordinal,
            entry,
            segment,
            record: None,
        }
    }

    pub(super) const fn ordinal(&self) -> usize {
        self.ordinal
    }

    pub(super) const fn physical_order(&self) -> (SegmentDigest, u64, usize) {
        (
            self.entry.segment_digest(),
            self.entry.record_offset(),
            self.ordinal,
        )
    }

    pub(super) fn into_bound(
        self,
    ) -> Result<(DecodedCatalogEntry, AdmittedSegmentRecord<'records>), CatalogAdmissionError> {
        let record = self
            .record
            .ok_or_else(|| CatalogAdmissionError::LocationNotTopLevel {
                identity: self.entry.identity(),
                segment_digest: self.entry.segment_digest(),
                record_offset: self.entry.record_offset(),
                record_length: self.entry.record_length().get(),
            })?;
        Ok((self.entry, record))
    }

    const fn segment_digest(&self) -> SegmentDigest {
        self.entry.segment_digest()
    }

    const fn record_offset(&self) -> u64 {
        self.entry.record_offset()
    }

    fn bind_if_length_matches(&mut self, offset: u64, record: AdmittedSegmentRecord<'records>) {
        let length_matches = record.header().record_length() == self.entry.record_length();
        if self.entry.record_offset() == offset && length_matches {
            self.record = Some(record);
        }
    }
}

pub(super) fn bind(
    entries: &mut [CatalogEntryPlan<'_, '_>],
    segments: &[&AdmittedSegment<'_>],
) -> Result<(), CatalogAdmissionError> {
    entries.sort_unstable_by_key(CatalogEntryPlan::physical_order);
    refuse_unreferenced_segments(entries, segments)?;
    for group in
        entries.chunk_by_mut(|first, second| first.segment_digest() == second.segment_digest())
    {
        bind_segment(group)?;
    }
    Ok(())
}

fn refuse_unreferenced_segments(
    entries: &[CatalogEntryPlan<'_, '_>],
    segments: &[&AdmittedSegment<'_>],
) -> Result<(), CatalogAdmissionError> {
    for segment in segments {
        let digest = segment.digest();
        if entries
            .binary_search_by_key(&digest, CatalogEntryPlan::segment_digest)
            .is_err()
        {
            return Err(CatalogAdmissionError::UnreferencedSegment { digest });
        }
    }
    Ok(())
}

fn bind_segment(entries: &mut [CatalogEntryPlan<'_, '_>]) -> Result<(), CatalogAdmissionError> {
    let Some(first) = entries.first() else {
        return Ok(());
    };
    let segment = first.segment;
    let digest = segment.digest();
    let mut pending = entries.iter_mut().peekable();
    let mut cursor = segment.record_cursor();
    while let Some(located) =
        cursor
            .next_record()
            .map_err(|source| CatalogAdmissionError::Segment {
                digest,
                source: Box::new(source),
            })?
    {
        skip_preceding_entries(&mut pending, located.offset);
        bind_entries_at_offset(&mut pending, located.offset, located.record);
    }
    cursor
        .finish()
        .map_err(|source| CatalogAdmissionError::Segment {
            digest,
            source: Box::new(source),
        })
}

fn skip_preceding_entries(
    entries: &mut std::iter::Peekable<std::slice::IterMut<'_, CatalogEntryPlan<'_, '_>>>,
    record_offset: u64,
) {
    while matches!(entries.peek(), Some(entry) if entry.record_offset() < record_offset) {
        let _skipped = entries.next();
    }
}

fn bind_entries_at_offset<'records>(
    entries: &mut std::iter::Peekable<std::slice::IterMut<'_, CatalogEntryPlan<'_, 'records>>>,
    record_offset: u64,
    record: AdmittedSegmentRecord<'records>,
) {
    while matches!(entries.peek(), Some(entry) if entry.record_offset() == record_offset) {
        if let Some(entry) = entries.next() {
            entry.bind_if_length_matches(record_offset, record);
        }
    }
}
