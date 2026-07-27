//! In-memory implementation of the semantic chunk-staging port.

use std::collections::BTreeMap;

use crate::{ChunkId, ReferenceStore};

use super::IngestionError;
use super::ingestion_error::IngestionAllocation;

pub(super) struct ReferenceChunkStaging<'a> {
    store: &'a ReferenceStore,
    chunks: BTreeMap<ChunkId, Box<[u8]>>,
    pending_bytes: usize,
}

impl<'a> ReferenceChunkStaging<'a> {
    pub(super) const fn new(store: &'a ReferenceStore) -> Self {
        Self {
            store,
            chunks: BTreeMap::new(),
            pending_bytes: 0,
        }
    }

    pub(super) fn into_parts(self) -> (BTreeMap<ChunkId, Box<[u8]>>, usize) {
        (self.chunks, self.pending_bytes)
    }

    pub(super) fn stage_chunk(
        &mut self,
        identity: ChunkId,
        bytes: &[u8],
    ) -> Result<(), IngestionError> {
        if let Some(existing) = self.store.chunks.get(&identity) {
            return compare_existing(identity, existing, bytes);
        }
        if let Some(existing) = self.chunks.get(&identity) {
            return compare_existing(identity, existing, bytes);
        }
        self.check_capacity(bytes.len())?;
        let owned = copy_chunk(bytes)?;
        self.pending_bytes = self
            .pending_bytes
            .checked_add(owned.len())
            .ok_or_else(|| capacity_error(self.store))?;
        self.chunks.insert(identity, owned.into_boxed_slice());
        Ok(())
    }
}

impl ReferenceChunkStaging<'_> {
    fn check_capacity(&self, incoming: usize) -> Result<(), IngestionError> {
        let attempted = self
            .store
            .materialized_bytes
            .checked_add(self.pending_bytes)
            .and_then(|value| value.checked_add(incoming))
            .ok_or_else(|| capacity_error(self.store))?;
        if attempted > self.store.capacity.get() {
            return Err(IngestionError::CapacityExceeded {
                capacity: self.store.capacity.get(),
                attempted,
            });
        }
        Ok(())
    }
}

fn copy_chunk(bytes: &[u8]) -> Result<Vec<u8>, IngestionError> {
    let mut owned = Vec::new();
    owned
        .try_reserve_exact(bytes.len())
        .map_err(|source| IngestionError::Allocation {
            target: IngestionAllocation::StagedChunk,
            requested: bytes.len(),
            source,
        })?;
    owned.extend_from_slice(bytes);
    Ok(owned)
}

fn compare_existing(
    identity: ChunkId,
    existing: &[u8],
    incoming: &[u8],
) -> Result<(), IngestionError> {
    if existing == incoming {
        return Ok(());
    }
    Err(IngestionError::ConflictingChunk { identity })
}

const fn capacity_error(store: &ReferenceStore) -> IngestionError {
    IngestionError::CapacityExceeded {
        capacity: store.capacity.get(),
        attempted: usize::MAX,
    }
}
