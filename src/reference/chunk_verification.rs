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
    #[cfg(test)]
    store.observed_chunk_hashes.borrow_mut().push(expected);
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

/// Fetches an already-authenticated immutable chunk for emission.
///
/// The verification pass hashed every selected chunk before the first byte
/// was written, and the in-memory view cannot change under `&self`, so the
/// emission pass looks the chunk up by identity and hashes nothing. A chunk
/// that vanished between the passes is impossible here; the arm exists so a
/// durable adapter that reuses this shape cannot forget it.
pub(super) fn emitted_chunk(
    store: &ReferenceStore,
    layout_id: LayoutId,
    index: usize,
    entry: LayoutEntry,
) -> Result<&[u8], ChunkVerificationError> {
    let expected = entry.chunk_id();
    store
        .chunk(expected)
        .ok_or(ChunkVerificationError::Missing {
            layout: layout_id,
            index,
            requested: expected,
        })
}
