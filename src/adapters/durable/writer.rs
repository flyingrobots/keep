//! This boundary module owns the durable writer: exclusive authority to
//! stage content into one version-two store in a single bounded pass and
//! publish it through the catalog protocol.

use std::io::Read;
use std::path::{Path, PathBuf};

use super::DurableIngestionError as Error;
use super::staged::DurableStagedBlob;
use super::writer_sink::DurableChunkSink;
use crate::adapters::{
    CatalogRestartPolicy, FilesystemCatalogPublisher, FilesystemCatalogSnapshot,
    FilesystemVersionTwoAdmission,
};
use crate::reference::ingest_stream;
use crate::{AdmittedLayout, BlobId, IngestionError, RegisteredStorageProfile, StagingLimits};

/// Exclusive authority to ingest into one pinned version-two root.
///
/// The writer holds the writer lock through its catalog publisher for its
/// lifetime. Staging reads the source once: each chunk is verified against
/// the pinned catalog by exact bytes and reused, or appended to a segment
/// stage as it is produced; the blob is never materialized. Commit publishes
/// the sealed segment and a catalog successor through the version-one
/// protocol, beside readers. Content becomes readable through
/// [`DurableSnapshot`](super::DurableSnapshot) by its committed layout at
/// once, and by blob identity once a retention root anchors it.
#[must_use]
pub struct DurableWriter {
    pub(super) publisher: FilesystemCatalogPublisher,
    pub(super) store_root: PathBuf,
    pub(super) policy: CatalogRestartPolicy,
}

impl DurableWriter {
    /// Pins one admitted version-two root for ingestion.
    ///
    /// # Errors
    ///
    /// Returns [`DurableIngestionError::Open`](Error::Open) when the
    /// publication directories cannot be pinned.
    pub fn open(
        admission: FilesystemVersionTwoAdmission,
        store_root: &Path,
        policy: CatalogRestartPolicy,
    ) -> Result<Self, Error> {
        let publisher = FilesystemCatalogPublisher::open_version_two(admission, policy)
            .map_err(|source| Error::Open { source })?;
        Ok(Self {
            publisher,
            store_root: store_root.to_path_buf(),
            policy,
        })
    }

    /// The store root.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.store_root
    }

    /// Stages `source` in one bounded pass without making anything visible.
    ///
    /// Memory beyond the streaming core's fixed scratch is the layout
    /// metadata bounded by `limits.entry_limit()` and the set of chunk
    /// identities written so far; chunk bytes go to the stage as they are
    /// produced. A refusal after the first new chunk leaves a
    /// `staging/current.seg` for the recovery protocol to discard, and a
    /// later staging refuses with `StageRetained` until it does.
    ///
    /// # Errors
    ///
    /// Returns [`DurableIngestionError`](Error) at the exact catalog,
    /// source, identity, limit, representation, or stage refusal.
    pub fn stage<R>(
        &mut self,
        source: &mut R,
        limits: StagingLimits,
    ) -> Result<DurableStagedBlob<'_>, Error>
    where
        R: Read + ?Sized,
    {
        let current = FilesystemCatalogSnapshot::load(&self.store_root, self.policy)
            .map_err(|source| Error::Catalog(Box::new(source)))?;
        let snapshot = current
            .snapshot()
            .map_err(|source| Error::Catalog(Box::new(source)))?;
        let generation = snapshot.generation();
        let mut sink = DurableChunkSink::new(&self.publisher, &snapshot);
        let (target, spans, logical_bytes) = ingest_stream(source, &mut sink, limits)?;
        let layout = AdmittedLayout::from_spans(
            target,
            RegisteredStorageProfile::FAST_CDC_64K_V1,
            spans,
            limits.entry_limit(),
        )
        .map_err(|source| Error::Ingestion(IngestionError::Layout(source)))?;
        let record = layout
            .encode_record()
            .map_err(|source| Error::Ingestion(IngestionError::LayoutEncoding(source)))?;
        let layout_id = record.id();
        sink.finish_layout(&record, layout_id)?;
        let (closed, accounting) = sink.seal()?;
        drop(snapshot);
        drop(current);
        Ok(DurableStagedBlob::new(
            self,
            (target, layout_id),
            generation,
            closed,
            accounting.with_logical_bytes(logical_bytes),
        ))
    }

    /// As [`Self::stage`], refusing when the complete source does not hash
    /// to `expected`. The stage written so far is left for recovery, as
    /// after any other staging refusal.
    ///
    /// # Errors
    ///
    /// As [`Self::stage`], plus
    /// [`IngestionError::BlobIdentityMismatch`] inside `Ingestion`.
    pub fn stage_expected<R>(
        &mut self,
        source: &mut R,
        expected: BlobId,
        limits: StagingLimits,
    ) -> Result<DurableStagedBlob<'_>, Error>
    where
        R: Read + ?Sized,
    {
        let staged = self.stage(source, limits)?;
        let observed = staged.target();
        if observed != expected {
            return Err(Error::Ingestion(IngestionError::BlobIdentityMismatch {
                expected,
                observed,
            }));
        }
        Ok(staged)
    }
}
