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

    pub(super) fn validate_for(&self, store: &ReferenceStore) -> Result<(), PublishError> {
        let existing = store.layout(self.layout_id);
        if let Some(layout) = existing
            && layout != &self.layout
        {
            return Err(PublishError::ConflictingLayout {
                identity: self.layout_id,
            });
        }
        let indexed = store
            .blob_layouts
            .get(&self.target())
            .is_some_and(|layouts| layouts.contains(&self.layout_id));
        match (existing, indexed) {
            (Some(_layout), false) => {
                return Err(PublishError::CommittedLayoutIndexMissing {
                    layout: self.layout_id,
                });
            }
            (None, true) => {
                return Err(PublishError::CommittedLayoutMissing {
                    layout: self.layout_id,
                });
            }
            (Some(layout), true) => {
                for entry in layout.entries() {
                    if store.chunk(entry.chunk_id()).is_none() {
                        return Err(PublishError::CommittedChunkMissing {
                            layout: self.layout_id,
                            chunk: entry.chunk_id(),
                        });
                    }
                }
            }
            (None, false) => {}
        }
        let mut attempted = store.materialized_bytes;
        for (identity, bytes) in &self.chunks {
            if let Some(existing) = store.chunks.get(identity) {
                if existing.as_ref() != bytes.as_ref() {
                    return Err(PublishError::ConflictingChunk {
                        identity: *identity,
                    });
                }
                continue;
            }
            attempted = attempted.checked_add(bytes.len()).ok_or_else(|| {
                PublishError::CapacityExceeded {
                    capacity: store.capacity.get(),
                    attempted: usize::MAX,
                }
            })?;
        }
        if attempted > store.capacity.get() {
            return Err(PublishError::CapacityExceeded {
                capacity: store.capacity.get(),
                attempted,
            });
        }
        Ok(())
    }

    pub(super) fn into_parts(self) -> (AdmittedLayout, BTreeMap<ChunkId, Box<[u8]>>) {
        (self.layout, self.chunks)
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::io::Cursor;

    use crate::{LayoutEntryLimit, PublishError, ReferenceStore, ReferenceStoreCapacity};

    #[test]
    fn committed_layout_with_a_missing_chunk_is_never_silently_repaired()
    -> Result<(), Box<dyn Error>> {
        let source = b"committed state cannot be repaired by ordinary publication";
        let mut store = ReferenceStore::new(ReferenceStoreCapacity::new(1_048_576));
        let mut first_source = Cursor::new(source);
        let published = store
            .stage(&mut first_source, LayoutEntryLimit::MAXIMUM)?
            .commit(&mut store)?;
        let missing = store
            .layout(published.layout_id())
            .and_then(|layout| layout.entries().first())
            .ok_or("published layout has no chunk")?
            .chunk_id();
        store.chunks.remove(&missing);
        let mut second_source = Cursor::new(source);
        let staged = store.stage(&mut second_source, LayoutEntryLimit::MAXIMUM)?;

        let error = staged
            .commit(&mut store)
            .err()
            .ok_or("ordinary publication silently repaired committed state")?;

        assert!(matches!(
            error,
            PublishError::CommittedChunkMissing {
                layout,
                chunk
            } if layout == published.layout_id() && chunk == missing
        ));
        assert!(!store.chunks.contains_key(&missing));
        Ok(())
    }

    #[test]
    fn committed_layout_without_its_blob_index_is_never_silently_repaired()
    -> Result<(), Box<dyn Error>> {
        let source = b"layout index loss requires an explicit recovery operation";
        let mut store = ReferenceStore::new(ReferenceStoreCapacity::new(1_048_576));
        let mut first_source = Cursor::new(source);
        let published = store
            .stage(&mut first_source, LayoutEntryLimit::MAXIMUM)?
            .commit(&mut store)?;
        let removed = store
            .blob_layouts
            .get_mut(&published.target())
            .is_some_and(|layouts| layouts.remove(&published.layout_id()));
        assert!(removed);
        let mut second_source = Cursor::new(source);
        let staged = store.stage(&mut second_source, LayoutEntryLimit::MAXIMUM)?;

        let error = staged
            .commit(&mut store)
            .err()
            .ok_or("ordinary publication silently repaired the blob index")?;

        assert!(matches!(
            error,
            PublishError::CommittedLayoutIndexMissing { layout }
                if layout == published.layout_id()
        ));
        assert!(!store.contains_blob(published.target()));
        Ok(())
    }

    #[test]
    fn committed_blob_index_without_its_layout_is_never_silently_repaired()
    -> Result<(), Box<dyn Error>> {
        let source = b"layout loss requires an explicit recovery operation";
        let mut store = ReferenceStore::new(ReferenceStoreCapacity::new(1_048_576));
        let mut first_source = Cursor::new(source);
        let published = store
            .stage(&mut first_source, LayoutEntryLimit::MAXIMUM)?
            .commit(&mut store)?;
        let removed = store.layouts.remove(&published.layout_id());
        assert!(removed.is_some());
        let mut second_source = Cursor::new(source);
        let staged = store.stage(&mut second_source, LayoutEntryLimit::MAXIMUM)?;

        let error = staged
            .commit(&mut store)
            .err()
            .ok_or("ordinary publication silently repaired the missing layout")?;

        assert!(matches!(
            error,
            PublishError::CommittedLayoutMissing { layout }
                if layout == published.layout_id()
        ));
        assert!(store.contains_blob(published.target()));
        assert!(store.layout(published.layout_id()).is_none());
        Ok(())
    }
}
