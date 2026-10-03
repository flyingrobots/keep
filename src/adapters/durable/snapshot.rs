//! This module owns one pinned durable view and the reads it answers.

use std::io::Write;
use std::path::Path;

use super::{
    DurableRangeReadReceipt, DurableReadError, DurableReconstructionReceipt, DurableStoreError,
    DurableView,
};
use crate::adapters::{
    AdmittedSegmentRecord, CatalogRestartPolicy, CatalogSnapshot, FilesystemRetentionSnapshot,
    LayoutDecodePolicy, ReaderAttemptLimit, SegmentRecordIdentity,
};
use crate::authenticated_read::{ChunkSource, read_admitted, reconstruct_admitted};
use crate::{AdmittedLayout, BlobId, ByteRange, ChunkId, LayoutId};

/// One consistent, fenced view of a version-two store.
///
/// The snapshot owns the shared reader fence for its lifetime, so no
/// collector can retire a segment it may read; every read borrows the
/// snapshot, so dropping the view mid-read is impossible. Blobs resolve
/// through the retained roots' anchors; layouts and chunks resolve through
/// the pinned catalog and are authenticated by the domain read cores
/// before a byte is emitted.
///
/// Opening materializes selected segment bytes within the aggregate segment-byte
/// limit in `CatalogRestartPolicy`. Catalog bytes are bounded separately by
/// [`crate::CatalogLength::MAXIMUM`]; decoded indexes allocate additionally under
/// format and record-count limits. The segment-byte limit is not a total
/// snapshot-memory cap, and this is not a lazy segment reader.
/// Retained roots are decoded and their closures verified
/// one at a time under their stored traversal limits. Reads decode one bounded
/// layout and stream authenticated chunks to the caller without assembling an
/// additional whole-blob buffer. Caller-owned output may allocate separately.
/// Each read also re-admits the already materialized catalog and segment bytes,
/// allocating bounded decoded indexes; even a short range pays that cost.
///
/// Blob lookup performs synchronous reads over the manifest-selected roots,
/// retaining at most one root's bytes and decoded anchors at a time. Its cost
/// is linear in those roots and anchors. A lookup rechecks canonical identity;
/// substituted or unreadable evidence fails rather than becoming absence.
/// The shared fence remains held across caller output callbacks so collection
/// cannot invalidate the read. No writer lock is acquired by these reads;
/// callers must not make output wait for an exclusive collector fence that
/// their own live snapshot prevents from being acquired.
#[must_use]
pub struct DurableSnapshot {
    view: FilesystemRetentionSnapshot,
    coordinates: DurableView,
    policy: CatalogRestartPolicy,
}

/// The pinned catalog as a chunk source: every chunk record's exact payload.
pub(super) struct CatalogChunks<'snapshot, 'head, 'catalog, 'records> {
    catalog: &'snapshot CatalogSnapshot<'head, 'catalog, 'records>,
}

impl<'snapshot, 'head, 'catalog, 'records> CatalogChunks<'snapshot, 'head, 'catalog, 'records> {
    pub(super) const fn new(
        catalog: &'snapshot CatalogSnapshot<'head, 'catalog, 'records>,
    ) -> Self {
        Self { catalog }
    }
}

impl ChunkSource for CatalogChunks<'_, '_, '_, '_> {
    fn chunk(&self, identity: ChunkId) -> Option<&[u8]> {
        self.catalog
            .record(SegmentRecordIdentity::Chunk(identity))
            .map(AdmittedSegmentRecord::payload)
    }
}

impl DurableSnapshot {
    /// Admits `store_root` as a version-two store, acquires the shared
    /// reader fence, double-collects one consistent view within `limit`
    /// attempts, and verifies every retained root's closure.
    ///
    /// The reader enforces the existing local writable, case-sensitive Linux
    /// ext4 profile without acquiring writer authority. The admitted directory
    /// capability is retained through view collection. Platform admission
    /// synchronizes the root directory before fencing; a synchronization
    /// failure is preserved under the snapshot's admission error.
    ///
    /// The aggregate byte limit in `policy` covers catalog-selected segment bytes only.
    /// Catalog bytes use [`crate::CatalogLength::MAXIMUM`] independently; decoded
    /// indexes and retention records allocate additionally under their format
    /// and record-count limits. Closure work is bounded per root by its persisted limits.
    /// This is blocking filesystem I/O and CPU verification, not publication;
    /// it performs no namespace writes and establishes no new durability.
    ///
    /// # Errors
    ///
    /// Returns [`DurableStoreError`] at the exact admission, fence,
    /// collection, or retained-root refusal.
    pub fn open(
        store_root: &Path,
        policy: CatalogRestartPolicy,
        limit: ReaderAttemptLimit,
    ) -> Result<Self, DurableStoreError> {
        let view = FilesystemRetentionSnapshot::load(store_root, policy, limit)
            .map_err(|source| DurableStoreError::Snapshot(Box::new(source)))?;
        let coordinates = DurableView::new(
            view.catalog().generation(),
            view.catalog().catalog_digest(),
            view.retention_head().copied(),
        );
        super::retained_anchors::verify(&view)?;
        Ok(Self {
            view,
            coordinates,
            policy,
        })
    }

    /// The exact view this snapshot pinned.
    #[must_use]
    pub const fn view(&self) -> DurableView {
        self.coordinates
    }

    /// Whether some retained root anchors `target` in this view.
    ///
    /// Scans the pinned manifest and admits one selected root at a time.
    ///
    /// # Errors
    ///
    /// Returns the exact selected-root read or admission failure.
    pub fn contains_blob(&self, target: BlobId) -> Result<bool, DurableStoreError> {
        super::retained_anchors::first_layout(&self.view, target).map(|layout| layout.is_some())
    }

    /// Reconstructs `target` through its canonically first retained anchor.
    ///
    /// # Errors
    ///
    /// Returns [`DurableReadError`] when no root anchors the blob, the view
    /// cannot be re-admitted, the layout refuses, or the reconstruction
    /// core refuses or its output fails.
    pub fn reconstruct<W>(
        &self,
        target: BlobId,
        output: &mut W,
    ) -> Result<DurableReconstructionReceipt, DurableReadError>
    where
        W: Write + ?Sized,
    {
        let layout_id = self.first_layout_id(target)?;
        self.reconstruct_layout(layout_id, output)
    }

    /// Reconstructs the exact committed layout `layout_id`, never another.
    ///
    /// # Errors
    ///
    /// As [`Self::reconstruct`], with `LayoutMissing` when the catalog names
    /// no such layout record.
    pub fn reconstruct_layout<W>(
        &self,
        layout_id: LayoutId,
        output: &mut W,
    ) -> Result<DurableReconstructionReceipt, DurableReadError>
    where
        W: Write + ?Sized,
    {
        let catalog = self.catalog()?;
        let layout = self.layout(&catalog, layout_id)?;
        let chunks = CatalogChunks::new(&catalog);
        reconstruct_admitted(&chunks, layout_id, &layout, output)
            .map(|receipt| DurableReconstructionReceipt::new(receipt, self.coordinates))
            .map_err(|source| DurableReadError::Reconstruction(Box::new(source.into())))
    }

    /// Reads exactly `requested` of `target` through its first retained
    /// anchor. The range receipt covers only overlapping chunks; snapshot
    /// and catalog admission also verify the surrounding stored evidence.
    ///
    /// # Errors
    ///
    /// As [`Self::reconstruct`], with the range core's refusals.
    pub fn read_range<W>(
        &self,
        target: BlobId,
        requested: ByteRange,
        output: &mut W,
    ) -> Result<DurableRangeReadReceipt, DurableReadError>
    where
        W: Write + ?Sized,
    {
        let layout_id = self.first_layout_id(target)?;
        self.read_layout_range(layout_id, requested, output)
    }

    /// Reads exactly `requested` through the exact committed layout.
    ///
    /// # Errors
    ///
    /// As [`Self::reconstruct_layout`], with the range core's refusals.
    pub fn read_layout_range<W>(
        &self,
        layout_id: LayoutId,
        requested: ByteRange,
        output: &mut W,
    ) -> Result<DurableRangeReadReceipt, DurableReadError>
    where
        W: Write + ?Sized,
    {
        let catalog = self.catalog()?;
        let layout = self.layout(&catalog, layout_id)?;
        let chunks = CatalogChunks::new(&catalog);
        read_admitted(&chunks, layout_id, &layout, requested, output)
            .map(|receipt| DurableRangeReadReceipt::new(receipt, self.coordinates))
            .map_err(|source| DurableReadError::RangeRead(Box::new(source.into())))
    }

    fn first_layout_id(&self, target: BlobId) -> Result<LayoutId, DurableReadError> {
        super::retained_anchors::first_layout(&self.view, target)
            .map_err(|source| DurableReadError::Retention(Box::new(source)))?
            .ok_or(DurableReadError::BlobMissing { requested: target })
    }

    pub(super) fn catalog(&self) -> Result<CatalogSnapshot<'_, '_, '_>, DurableReadError> {
        self.view
            .catalog()
            .snapshot()
            .map_err(|source| DurableReadError::View(Box::new(source)))
    }

    fn layout(
        &self,
        catalog: &CatalogSnapshot<'_, '_, '_>,
        layout_id: LayoutId,
    ) -> Result<AdmittedLayout, DurableReadError> {
        let record = catalog
            .record(SegmentRecordIdentity::Layout(layout_id))
            .ok_or(DurableReadError::LayoutMissing {
                requested: layout_id,
            })?;
        let policy = LayoutDecodePolicy::new(self.policy.segment_read().layout_entry_limit())
            .with_expected_id(layout_id);
        AdmittedLayout::decode_record(record.payload(), policy)
            .map_err(DurableReadError::LayoutDecode)
    }
}
