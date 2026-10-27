//! In-memory writer-authorized storage that records every port call.

use std::io;

use keep::{
    CatalogPublicationPhase, RetainedCatalogPublicationStorage, RetainedClosureAdmission,
    RetentionHead, RetentionManifestEntry,
};

use crate::recording_storage::RecordingStorage;
use crate::retention_fixture::RetentionRecords;

/// One retention evidence read the gate requested.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvidenceRead {
    /// `read_retention_head`.
    Head,
    /// `read_retention_manifest`.
    Manifest,
    /// `read_retained_root` for one exact entry.
    Root(RetentionManifestEntry),
}

/// Storage model over exact retention records and recorded catalog stages.
pub struct ModelStorage {
    stages: RecordingStorage,
    records: RetentionRecords,
    later_heads: Vec<Option<Vec<u8>>>,
    failing_head_read: Option<usize>,
    reads: Vec<(EvidenceRead, usize)>,
}

impl ModelStorage {
    /// Serves `records` and admits every catalog stage.
    pub const fn new(records: RetentionRecords) -> Self {
        Self::with_stages(records, RecordingStorage::succeeding())
    }

    /// Serves `records` through the supplied catalog stage recorder.
    pub const fn with_stages(records: RetentionRecords, stages: RecordingStorage) -> Self {
        Self {
            stages,
            records,
            later_heads: Vec::new(),
            failing_head_read: None,
            reads: Vec::new(),
        }
    }

    /// Serves `head` on every head read after the first.
    #[must_use]
    pub fn then_head(mut self, head: Option<Vec<u8>>) -> Self {
        self.later_heads.push(head);
        self
    }

    /// Fails the zero-based `ordinal` head read.
    #[must_use]
    pub const fn failing_head_read(mut self, ordinal: usize) -> Self {
        self.failing_head_read = Some(ordinal);
        self
    }

    /// Returns every catalog stage attempted.
    pub fn stages(&self) -> &[CatalogPublicationPhase] {
        self.stages.observed()
    }

    /// Returns every evidence read with the catalog stage count before it.
    pub fn reads(&self) -> &[(EvidenceRead, usize)] {
        &self.reads
    }

    fn record(&mut self, read: EvidenceRead) {
        self.reads.push((read, self.stages.observed().len()));
    }

    fn head_reads(&self) -> usize {
        self.reads
            .iter()
            .filter(|(read, _)| *read == EvidenceRead::Head)
            .count()
    }
}

impl RetainedCatalogPublicationStorage for ModelStorage {
    type Stages = RecordingStorage;

    fn read_retention_head(&mut self) -> io::Result<Option<Vec<u8>>> {
        let ordinal = self.head_reads();
        self.record(EvidenceRead::Head);
        if self.failing_head_read == Some(ordinal) {
            return Err(io::Error::other("injected retention-head read failure"));
        }
        let later = ordinal.checked_sub(1).and_then(|index| {
            self.later_heads
                .get(index)
                .or_else(|| self.later_heads.last())
        });
        Ok(later.map_or_else(|| self.records.head.clone(), Clone::clone))
    }

    fn read_retention_manifest(&mut self, head: &RetentionHead) -> io::Result<Vec<u8>> {
        self.record(EvidenceRead::Manifest);
        match &self.records.manifest {
            Some((digest, bytes)) if *digest == head.manifest_digest() => Ok(bytes.clone()),
            _ => Err(io::Error::from(io::ErrorKind::NotFound)),
        }
    }

    fn read_retained_root(&mut self, entry: RetentionManifestEntry) -> io::Result<Vec<u8>> {
        self.record(EvidenceRead::Root(entry));
        self.records
            .roots
            .iter()
            .find(|(digest, _)| *digest == entry.root_digest())
            .map(|(_, bytes)| bytes.clone())
            .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))
    }

    fn catalog_stages(&mut self, _admission: &RetainedClosureAdmission) -> &mut RecordingStorage {
        &mut self.stages
    }
}
