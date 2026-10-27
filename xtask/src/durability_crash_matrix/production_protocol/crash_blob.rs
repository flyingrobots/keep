//! This module owns the blobs crash children ingest through the public writer.
//!
//! A child stages exactly these records, and restart reconstructs exactly
//! these bytes, so both sides derive each blob from one definition.

use keep::{
    AdmittedLayout, AdmittedSegmentRecord, BlobId, CanonicalLayoutRecord, ChunkSpan, FastCdc,
    LayoutEntryLimit, RegisteredStorageProfile, RetentionAnchor,
};

use super::super::DurabilityCrashMatrixError;
use super::verification;

/// The exact bytes the version-two writer ingests before process death.
pub(in crate::durability_crash_matrix) const VERSION_TWO_CONTENT: &[u8] =
    b"keep version-two crash campaign: one blob ingested after migration";

/// The synthetic source byte of the frozen one-zero bundle corpus.
const BUNDLE_CONTENT: &[u8] = &[0];

/// One blob with its canonical layout and chunk spans.
pub(in crate::durability_crash_matrix) struct CrashBlob {
    content: &'static [u8],
    spans: Vec<ChunkSpan>,
    record: CanonicalLayoutRecord,
    target: BlobId,
}

impl CrashBlob {
    /// The blob the version-two writer ingests before process death.
    pub(in crate::durability_crash_matrix) fn version_two()
    -> Result<Self, DurabilityCrashMatrixError> {
        Self::identify(VERSION_TWO_CONTENT)
    }

    /// The blob whose first catalog is the frozen bundle corpus.
    pub(in crate::durability_crash_matrix) fn bundle() -> Result<Self, DurabilityCrashMatrixError> {
        Self::identify(BUNDLE_CONTENT)
    }

    fn identify(content: &'static [u8]) -> Result<Self, DurabilityCrashMatrixError> {
        let target = BlobId::hash_bytes(content)
            .map_err(|source| verification("identify crash blob", source))?;
        let mut detector = FastCdc::new();
        let mut spans = Vec::new();
        detector
            .feed(content, |span| spans.push(span))
            .map_err(|source| verification("chunk crash blob", source))?;
        if let Some(span) = detector
            .finish()
            .map_err(|source| verification("finish crash blob", source))?
        {
            spans.push(span);
        }
        let layout = AdmittedLayout::from_spans(
            target,
            RegisteredStorageProfile::FAST_CDC_64K_V1,
            spans.clone(),
            LayoutEntryLimit::MAXIMUM,
        )
        .map_err(|source| verification("admit crash layout", source))?;
        let record = layout
            .encode_record()
            .map_err(|source| verification("encode crash layout", source))?;
        Ok(Self {
            content,
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
                .map_err(|source| verification("bound crash chunk", source))?;
            let end = usize::try_from(span.end().get())
                .map_err(|source| verification("bound crash chunk", source))?;
            let bytes = self
                .content
                .get(start..end)
                .ok_or(DurabilityCrashMatrixError::FixtureRange)?;
            records.push(
                AdmittedSegmentRecord::for_chunk(bytes)
                    .map_err(|source| verification("admit crash chunk", source))?,
            );
        }
        records.push(
            AdmittedSegmentRecord::for_layout(&self.record)
                .map_err(|source| verification("admit crash layout", source))?,
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
