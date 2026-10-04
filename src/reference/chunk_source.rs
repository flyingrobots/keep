//! This module owns the reference adapter implementation of immutable chunk lookup.

use crate::ChunkId;
use crate::authenticated_read::ChunkSource;

impl ChunkSource for crate::ReferenceStore {
    fn chunk(&self, identity: ChunkId) -> Option<&[u8]> {
        self.chunk(identity)
    }

    fn note_chunk_hash(&self, identity: ChunkId) {
        #[cfg(test)]
        self.observed_chunk_hashes.borrow_mut().push(identity);
        #[cfg(not(test))]
        let _ = identity;
    }
}
