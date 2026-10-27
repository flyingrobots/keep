//! This module owns the one blob the version-two crash campaign ingests.
//!
//! The child stages exactly these records, and restart reconstructs exactly
//! these bytes, so both sides derive the blob from one definition.

use keep::{
    AdmittedLayout, AdmittedSegmentRecord, BlobId, CanonicalLayoutRecord, ChunkSpan, FastCdc,
    LayoutEntryLimit, RegisteredStorageProfile, RetentionAnchor,
};

use super::super::DurabilityCrashMatrixError;
use super::verification;

/// The exact bytes the version-two writer ingests before process death.
pub(in crate::durability_crash_matrix) const CONTENT: &[u8] =
    b"keep version-two crash campaign: one blob ingested after migration";

/// The canonical layout and chunk spans of [`CONTENT`].
pub(in crate::durability_crash_matrix) struct VersionTwoBlob {
    spans: Vec<ChunkSpan>,
    record: CanonicalLayoutRecord,
    target: BlobId,
}

impl VersionTwoBlob {
    pub(in crate::durability_crash_matrix) fn identify() -> Result<Self, DurabilityCrashMatrixError>
    {
        let target = BlobId::hash_bytes(CONTENT)
            .map_err(|source| verification("identify version-two crash blob", source))?;
        let mut detector = FastCdc::new();
        let mut spans = Vec::new();
        detector
            .feed(CONTENT, |span| spans.push(span))
            .map_err(|source| verification("chunk version-two crash blob", source))?;
        if let Some(span) = detector
            .finish()
            .map_err(|source| verification("finish version-two crash blob", source))?
        {
            spans.push(span);
        }
        let layout = AdmittedLayout::from_spans(
            target,
            RegisteredStorageProfile::FAST_CDC_64K_V1,
            spans.clone(),
            LayoutEntryLimit::MAXIMUM,
        )
        .map_err(|source| verification("admit version-two crash layout", source))?;
        let record = layout
            .encode_record()
            .map_err(|source| verification("encode version-two crash layout", source))?;
        Ok(Self {
            spans,
            record,
            target,
        })
    }

    /// The chunk records followed by the layout record, in staging order.
    pub(in crate::durability_crash_matrix) fn records(
        &self,
    ) -> Result<Vec<AdmittedSegmentRecord<'_>>, DurabilityCrashMatrixError> {
        let mut records = Vec::with_capacity(self.spans.len());
        for span in &self.spans {
            let start = usize::try_from(span.offset().get())
                .map_err(|source| verification("bound version-two crash chunk", source))?;
            let end = usize::try_from(span.end().get())
                .map_err(|source| verification("bound version-two crash chunk", source))?;
            let bytes = CONTENT
                .get(start..end)
                .ok_or(DurabilityCrashMatrixError::FixtureRange)?;
            records.push(
                AdmittedSegmentRecord::for_chunk(bytes)
                    .map_err(|source| verification("admit version-two crash chunk", source))?,
            );
        }
        records.push(
            AdmittedSegmentRecord::for_layout(&self.record)
                .map_err(|source| verification("admit version-two crash layout", source))?,
        );
        Ok(records)
    }

    pub(in crate::durability_crash_matrix) const fn target(&self) -> BlobId {
        self.target
    }

    pub(in crate::durability_crash_matrix) const fn anchor(&self) -> RetentionAnchor {
        RetentionAnchor::new(self.target, self.record.id())
    }
}
