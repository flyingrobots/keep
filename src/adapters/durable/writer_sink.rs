//! This module owns the durable chunk sink: each chunk the streaming core
//! hands over is either verified against the pinned catalog by exact bytes
//! or appended to a lazily created segment stage, never held in memory.

use std::collections::BTreeSet;

use super::DurableIngestionError as Error;
use super::ingestion_receipt::IngestionAccounting;
use crate::adapters::filesystem_catalog_publisher::ClosedSelection;
use crate::adapters::{
    AdmittedSegmentRecord, CatalogSnapshot, FilesystemCatalogPublisher, FilesystemSegmentStage,
    SegmentRecordIdentity, SegmentRecordLimit, SegmentStageCreateError, StagedSegment,
};
use crate::reference::ChunkSink;
use crate::{CanonicalLayoutRecord, ChunkId, LayoutId};

pub(super) struct DurableChunkSink<'publisher, 'catalog> {
    publisher: &'publisher FilesystemCatalogPublisher,
    catalog: &'catalog CatalogSnapshot<'catalog, 'catalog, 'catalog>,
    staged: Option<StagedSegment<FilesystemSegmentStage<'publisher>>>,
    written: BTreeSet<ChunkId>,
    accounting: IngestionAccounting,
}

impl<'publisher, 'catalog> DurableChunkSink<'publisher, 'catalog> {
    pub(super) const fn new(
        publisher: &'publisher FilesystemCatalogPublisher,
        catalog: &'catalog CatalogSnapshot<'catalog, 'catalog, 'catalog>,
    ) -> Self {
        Self {
            publisher,
            catalog,
            staged: None,
            written: BTreeSet::new(),
            accounting: IngestionAccounting::EMPTY,
        }
    }

    /// Appends the layout record unless the catalog already holds it
    /// byte-identically.
    pub(super) fn finish_layout(
        &mut self,
        record: &CanonicalLayoutRecord,
        identity: LayoutId,
    ) -> Result<(), Error> {
        if let Some(existing) = self.catalog.record(SegmentRecordIdentity::Layout(identity)) {
            if existing.payload() == record.bytes() {
                return Ok(());
            }
            return Err(Error::LayoutRepresentation { identity });
        }
        let admitted = AdmittedSegmentRecord::for_layout(record).map_err(Error::Record)?;
        self.append(admitted)
    }

    /// Seals the stage when one was created and closes it into a selection
    /// input that no longer borrows the publisher.
    pub(super) fn seal(self) -> Result<(Option<ClosedSelection>, IngestionAccounting), Error> {
        let Some(staged) = self.staged else {
            return Ok((None, self.accounting));
        };
        let sealed = staged
            .seal()
            .map_err(|source| Error::Stage(Box::new(source)))?;
        let closed = self
            .publisher
            .close_sealed(sealed)
            .map_err(Error::Selection)?;
        Ok((Some(closed), self.accounting))
    }

    fn append(&mut self, record: AdmittedSegmentRecord<'_>) -> Result<(), Error> {
        let staged = if let Some(staged) = self.staged.take() {
            staged
        } else {
            let stage = self.publisher.create_segment_stage().map_err(|source| {
                let SegmentStageCreateError::Create { source } = source;
                if source.kind() == std::io::ErrorKind::AlreadyExists {
                    Error::StageRetained
                } else {
                    Error::Open { source }
                }
            })?;
            StagedSegment::begin(stage, SegmentRecordLimit::MAXIMUM)
                .map_err(|source| Error::Stage(Box::new(source)))?
        };
        self.staged = Some(
            staged
                .append(record)
                .map_err(|source| Error::Stage(Box::new(source)))?,
        );
        Ok(())
    }
}

impl ChunkSink for DurableChunkSink<'_, '_> {
    type Error = Error;

    fn stage_chunk(&mut self, identity: ChunkId, bytes: &[u8]) -> Result<(), Error> {
        let length = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
        if let Some(existing) = self.catalog.record(SegmentRecordIdentity::Chunk(identity)) {
            if existing.payload() != bytes {
                return Err(Error::ChunkRepresentation { identity });
            }
            self.accounting.reused_chunk(length);
            return Ok(());
        }
        if self.written.contains(&identity) {
            self.accounting.reused_chunk(length);
            return Ok(());
        }
        let record = AdmittedSegmentRecord::for_chunk(bytes).map_err(Error::Record)?;
        self.append(record)?;
        let _inserted = self.written.insert(identity);
        self.accounting.new_chunk(length);
        Ok(())
    }
}
