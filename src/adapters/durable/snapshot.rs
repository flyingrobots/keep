//! This module owns one pinned durable view and the reads it answers.

use std::collections::{BTreeMap, BTreeSet};
use std::io::{self, Write};
use std::path::Path;

use super::{
    DurableRangeReadReceipt, DurableReadError, DurableReconstructionReceipt, DurableStoreError,
    DurableView,
};
use crate::adapters::retention::AdmittedRetentionRoot;
use crate::adapters::{
    AdmittedSegmentRecord, CatalogRestartPolicy, CatalogSnapshot, FilesystemRetentionSnapshot,
    GcRetentionState, LayoutDecodePolicy, ReaderAttemptLimit, SegmentRecordIdentity,
};
use crate::reference::{ChunkSource, read_admitted, reconstruct_admitted};
use crate::{AdmittedLayout, BlobId, ByteRange, ChunkId, LayoutId};

/// One consistent, fenced view of a version-two store.
///
/// The snapshot owns the shared reader fence for its lifetime, so no
/// collector can retire a segment it may read; every read borrows the
/// snapshot, so dropping the view mid-read is impossible. Blobs resolve
/// through the retained roots' anchors; layouts and chunks resolve through
/// the pinned catalog and are authenticated by the reference read cores
/// before a byte is emitted.
#[must_use]
pub struct DurableSnapshot {
    view: FilesystemRetentionSnapshot,
    coordinates: DurableView,
    anchors: BTreeMap<BlobId, BTreeSet<LayoutId>>,
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
    /// attempts, and indexes every retained root's anchors.
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
        let retention = view
            .retention_head()
            .map_or(GcRetentionState::Empty, |head| {
                GcRetentionState::Published {
                    generation: head.generation(),
                    manifest_digest: head.manifest_digest(),
                }
            });
        let coordinates = DurableView::new(
            view.catalog().generation(),
            view.catalog().catalog_digest(),
            retention,
        );
        let anchors = anchors(&view)?;
        Ok(Self {
            view,
            coordinates,
            anchors,
            policy,
        })
    }

    /// The exact view this snapshot pinned.
    #[must_use]
    pub const fn view(&self) -> DurableView {
        self.coordinates
    }

    /// Whether some retained root anchors `target` in this view.
    #[must_use]
    pub fn contains_blob(&self, target: BlobId) -> bool {
        self.anchors.contains_key(&target)
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
        let chunks = CatalogChunks { catalog: &catalog };
        reconstruct_admitted(&chunks, layout_id, &layout, output)
            .map(|receipt| DurableReconstructionReceipt::new(receipt, self.coordinates))
            .map_err(|source| DurableReadError::Reconstruction(Box::new(source)))
    }

    /// Reads exactly `requested` of `target` through its first retained
    /// anchor, authenticating only the overlapping chunks.
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
        let chunks = CatalogChunks { catalog: &catalog };
        read_admitted(&chunks, layout_id, &layout, requested, output)
            .map(|receipt| DurableRangeReadReceipt::new(receipt, self.coordinates))
            .map_err(|source| DurableReadError::RangeRead(Box::new(source)))
    }

    fn first_layout_id(&self, target: BlobId) -> Result<LayoutId, DurableReadError> {
        self.anchors
            .get(&target)
            .and_then(|layouts| layouts.first().copied())
            .ok_or(DurableReadError::BlobMissing { requested: target })
    }

    pub(super) fn catalog(&self) -> Result<CatalogSnapshot<'_, '_, '_>, DurableReadError> {
        self.view
            .catalog()
            .snapshot()
            .map_err(|source| DurableReadError::View(Box::new(source)))
    }

    pub(super) fn layout(
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

/// Every retained root's anchors, blob to its committed layouts.
fn anchors(
    view: &FilesystemRetentionSnapshot,
) -> Result<BTreeMap<BlobId, BTreeSet<LayoutId>>, DurableStoreError> {
    let mut anchors: BTreeMap<BlobId, BTreeSet<LayoutId>> = BTreeMap::new();
    let Some(manifest) = view.manifest() else {
        return Ok(anchors);
    };
    for entry in manifest.entries() {
        let namespace = entry.namespace();
        let refused = |source: io::Error| DurableStoreError::RetainedRoot { namespace, source };
        let bytes = view
            .retained_root(namespace)
            .map_err(|source| refused(io::Error::other(source)))?
            .ok_or_else(|| refused(io::Error::other("the selected root is absent")))?;
        let root = AdmittedRetentionRoot::decode(&bytes)
            .map_err(|source| refused(io::Error::new(io::ErrorKind::InvalidData, source)))?;
        for anchor in root.root().anchors() {
            anchors
                .entry(anchor.blob_id())
                .or_default()
                .insert(anchor.layout_id());
        }
    }
    Ok(anchors)
}
