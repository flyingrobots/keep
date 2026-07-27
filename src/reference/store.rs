//! Visible state of the non-durable reference adapter.

use std::collections::{BTreeMap, BTreeSet};

use crate::{AdmittedLayout, BlobId, ChunkId, LayoutId};

use super::{PublishError, PublishedBlob, ReferenceStoreCapacity, StagedBlob};

/// Capacity-bounded, in-memory, non-durable reference backend.
///
/// Exact chunks are deduplicated by [`ChunkId`]. Deduplication is only a
/// storage fact: this adapter exposes no retention or durability claim.
pub struct ReferenceStore {
    pub(super) capacity: ReferenceStoreCapacity,
    pub(super) chunks: BTreeMap<ChunkId, Box<[u8]>>,
    layouts: BTreeMap<LayoutId, AdmittedLayout>,
    blob_layouts: BTreeMap<BlobId, BTreeSet<LayoutId>>,
    pub(super) materialized_bytes: usize,
}

impl ReferenceStore {
    /// Constructs an empty non-durable reference backend.
    #[must_use]
    pub const fn new(capacity: ReferenceStoreCapacity) -> Self {
        Self {
            capacity,
            chunks: BTreeMap::new(),
            layouts: BTreeMap::new(),
            blob_layouts: BTreeMap::new(),
            materialized_bytes: 0,
        }
    }

    /// Returns whether at least one committed layout names this blob.
    ///
    /// Presence is not retention or durability.
    #[must_use]
    pub fn contains_blob(&self, target: BlobId) -> bool {
        self.blob_layouts
            .get(&target)
            .is_some_and(|layouts| !layouts.is_empty())
    }

    pub(super) fn publish(&mut self, staged: StagedBlob) -> Result<PublishedBlob, PublishError> {
        staged.validate_for(self)?;
        let target = staged.target();
        let layout_id = staged.layout_id();
        let (layout, chunks) = staged.into_parts();
        for (identity, bytes) in chunks {
            if self.chunks.contains_key(&identity) {
                continue;
            }
            self.materialized_bytes = self
                .materialized_bytes
                .checked_add(bytes.len())
                .ok_or_else(|| PublishError::CapacityExceeded {
                    capacity: self.capacity.get(),
                    attempted: usize::MAX,
                })?;
            self.chunks.insert(identity, bytes);
        }
        self.layouts.entry(layout_id).or_insert(layout);
        self.blob_layouts
            .entry(target)
            .or_default()
            .insert(layout_id);
        Ok(PublishedBlob::new(target, layout_id))
    }

    pub(super) fn layout(&self, identity: LayoutId) -> Option<&AdmittedLayout> {
        self.layouts.get(&identity)
    }

    pub(super) fn first_layout_id(&self, target: BlobId) -> Option<LayoutId> {
        self.blob_layouts
            .get(&target)
            .and_then(BTreeSet::first)
            .copied()
    }

    pub(super) fn chunk(&self, identity: ChunkId) -> Option<&[u8]> {
        self.chunks.get(&identity).map(Box::as_ref)
    }
}
