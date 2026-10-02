//! This module owns exact chunk authentication through an immutable semantic source.

use crate::{ChunkHashError, ChunkId, LayoutEntry, LayoutId};

/// One admitted view's exact chunk lookup: the reference store's in-memory
/// map, or a durable snapshot's fenced catalog. Every read core hashes what
/// the source returns before trusting it.
/// Implementations must expose immutable bytes for the entire read operation:
/// emission deliberately reuses the verification pass without rehashing.
pub(crate) trait ChunkSource {
    /// The exact bytes stored under `identity`, if the view holds them.
    fn chunk(&self, identity: ChunkId) -> Option<&[u8]>;

    /// Records that `identity` was hashed, for laws over the hashing pass.
    fn note_chunk_hash(&self, identity: ChunkId) {
        let _ = identity;
    }
}

pub(super) fn verified_chunk<S: ChunkSource + ?Sized>(
    store: &S,
    layout_id: LayoutId,
    index: usize,
    entry: LayoutEntry,
) -> Result<&[u8], ChunkVerificationError> {
    let expected = entry.chunk_id();
    let bytes = ChunkSource::chunk(store, expected).ok_or(ChunkVerificationError::Missing {
        layout: layout_id,
        index,
        requested: expected,
    })?;
    store.note_chunk_hash(expected);
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
pub(crate) enum ChunkVerificationError {
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
pub(super) fn emitted_chunk<S: ChunkSource + ?Sized>(
    store: &S,
    layout_id: LayoutId,
    index: usize,
    entry: LayoutEntry,
) -> Result<&[u8], ChunkVerificationError> {
    let expected = entry.chunk_id();
    ChunkSource::chunk(store, expected).ok_or(ChunkVerificationError::Missing {
        layout: layout_id,
        index,
        requested: expected,
    })
}
