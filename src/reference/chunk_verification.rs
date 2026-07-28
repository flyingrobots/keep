//! Exact reference-store chunk lookup and authentication.

use crate::{ChunkHashError, ChunkId, LayoutEntry, LayoutId, ReferenceStore};

pub(super) fn verified_chunk(
    store: &ReferenceStore,
    layout_id: LayoutId,
    index: usize,
    entry: LayoutEntry,
) -> Result<&[u8], ChunkVerificationError> {
    let expected = entry.chunk_id();
    let bytes = store
        .chunk(expected)
        .ok_or(ChunkVerificationError::Missing {
            layout: layout_id,
            index,
            requested: expected,
        })?;
    let observed = ChunkId::hash_bytes(bytes).map_err(|source| ChunkVerificationError::Hash {
        layout: layout_id,
        index,
        expected,
        source,
    })?;
    if observed != expected {
        return Err(ChunkVerificationError::IdentityMismatch {
            layout: layout_id,
            index,
            expected,
            observed,
        });
    }
    Ok(bytes)
}

#[derive(Clone, Copy, Debug)]
pub(super) enum ChunkVerificationError {
    Missing {
        layout: LayoutId,
        index: usize,
        requested: ChunkId,
    },
    Hash {
        layout: LayoutId,
        index: usize,
        expected: ChunkId,
        source: ChunkHashError,
    },
    IdentityMismatch {
        layout: LayoutId,
        index: usize,
        expected: ChunkId,
        observed: ChunkId,
    },
}
