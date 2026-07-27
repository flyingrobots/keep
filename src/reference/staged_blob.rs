//! Owned work awaiting an explicit reference-store publication.

use std::collections::BTreeMap;

use crate::{AdmittedLayout, BlobId, ChunkId, LayoutId};

use super::{PublishError, PublishedBlob, ReferenceStore};

/// Validated reference-store work that is not yet visible.
///
/// This value explicitly owns every new unique chunk byte required by its
/// target. Its memory may therefore grow with logical blob length, bounded by
/// the store capacity checked during staging. This materialization belongs to
/// the deliberately in-memory reference adapter, not to the streaming core's
/// scratch state.
#[must_use = "staged work remains invisible until commit is called"]
pub struct StagedBlob {
    layout: AdmittedLayout,
    layout_id: LayoutId,
    chunks: BTreeMap<ChunkId, Box<[u8]>>,
    pending_bytes: usize,
}

impl StagedBlob {
    pub(super) const fn new(
        layout: AdmittedLayout,
        layout_id: LayoutId,
        chunks: BTreeMap<ChunkId, Box<[u8]>>,
        pending_bytes: usize,
    ) -> Self {
        Self {
            layout,
            layout_id,
            chunks,
            pending_bytes,
        }
    }

    /// Returns the exact logical target.
    #[must_use]
    pub const fn target(&self) -> BlobId {
        self.layout.target()
    }

    /// Returns the canonical layout identity.
    #[must_use]
    pub const fn layout_id(&self) -> LayoutId {
        self.layout_id
    }

    /// Returns the validated semantic layout.
    #[must_use]
    pub const fn layout(&self) -> &AdmittedLayout {
        &self.layout
    }

    /// Returns the number of new unique chunk values materialized by staging.
    #[must_use]
    pub fn pending_chunk_count(&self) -> usize {
        self.chunks.len()
    }

    /// Returns the exact new chunk bytes materialized by staging.
    ///
    /// This excludes chunks already present in the destination store.
    #[must_use]
    pub const fn pending_materialized_bytes(&self) -> usize {
        self.pending_bytes
    }

    /// Explicitly makes the staged blob visible in `store`.
    ///
    /// This consumes the staged work. The non-durable reference adapter makes
    /// the transition atomically with respect to its synchronous `&mut`
    /// access; it makes no crash or power-loss claim.
    ///
    /// # Errors
    ///
    /// Returns [`PublishError`] if intervening work exhausted capacity or
    /// introduced conflicting bytes or layout state.
    pub fn commit(self, store: &mut ReferenceStore) -> Result<PublishedBlob, PublishError> {
        store.publish(self)
    }

    pub(super) fn validate_for(&self, store: &ReferenceStore) -> Result<usize, PublishError> {
        validate_committed_state(self, store)?;
        validate_required_chunks(self, store)?;
        validate_pending_chunks(self, store)
    }

    pub(super) fn into_parts(self) -> (AdmittedLayout, BTreeMap<ChunkId, Box<[u8]>>) {
        (self.layout, self.chunks)
    }
}

fn validate_pending_chunks(
    staged: &StagedBlob,
    store: &ReferenceStore,
) -> Result<usize, PublishError> {
    let mut attempted = store.materialized_bytes;
    for (identity, bytes) in &staged.chunks {
        if let Some(existing) = store.chunks.get(identity) {
            if existing.as_ref() != bytes.as_ref() {
                return Err(PublishError::ConflictingChunk {
                    identity: *identity,
                });
            }
            continue;
        }
        attempted =
            attempted
                .checked_add(bytes.len())
                .ok_or_else(|| PublishError::CapacityExceeded {
                    capacity: store.capacity.get(),
                    attempted: usize::MAX,
                })?;
    }
    if attempted > store.capacity.get() {
        return Err(PublishError::CapacityExceeded {
            capacity: store.capacity.get(),
            attempted,
        });
    }
    Ok(attempted)
}

fn validate_required_chunks(
    staged: &StagedBlob,
    store: &ReferenceStore,
) -> Result<(), PublishError> {
    for entry in staged.layout.entries() {
        let identity = entry.chunk_id();
        if !store.chunks.contains_key(&identity) && !staged.chunks.contains_key(&identity) {
            return Err(PublishError::StagedChunkMissing {
                layout: staged.layout_id,
                chunk: identity,
            });
        }
    }
    Ok(())
}

fn validate_committed_state(
    staged: &StagedBlob,
    store: &ReferenceStore,
) -> Result<(), PublishError> {
    let existing = store.layout(staged.layout_id);
    if let Some(layout) = existing
        && layout != &staged.layout
    {
        return Err(PublishError::ConflictingLayout {
            identity: staged.layout_id,
        });
    }
    if let Some(observed) = store.first_other_index(staged.layout_id, staged.target()) {
        return Err(PublishError::CommittedLayoutMisindexed {
            layout: staged.layout_id,
            observed,
        });
    }
    validate_layout_index(staged, store, existing)
}

fn validate_layout_index(
    staged: &StagedBlob,
    store: &ReferenceStore,
    existing: Option<&AdmittedLayout>,
) -> Result<(), PublishError> {
    let indexed = store
        .blob_layouts
        .get(&staged.target())
        .is_some_and(|layouts| layouts.contains(&staged.layout_id));
    match (existing, indexed) {
        (Some(_layout), false) => Err(PublishError::CommittedLayoutIndexMissing {
            layout: staged.layout_id,
        }),
        (None, true) => Err(PublishError::CommittedLayoutMissing {
            layout: staged.layout_id,
        }),
        (Some(layout), true) => validate_committed_chunks(staged.layout_id, layout, store),
        (None, false) => Ok(()),
    }
}

fn validate_committed_chunks(
    layout_id: LayoutId,
    layout: &AdmittedLayout,
    store: &ReferenceStore,
) -> Result<(), PublishError> {
    for entry in layout.entries() {
        if store.chunk(entry.chunk_id()).is_none() {
            return Err(PublishError::CommittedChunkMissing {
                layout: layout_id,
                chunk: entry.chunk_id(),
            });
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "staged_blob_tests.rs"]
mod tests;
