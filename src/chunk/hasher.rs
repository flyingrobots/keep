//! One-pass physical chunk identity calculation.

use super::id::ChunkId;
use super::length::ChunkLength;

const DATA_MAGIC: &[u8; 16] = b"KEEP:CHUNK:DATA\0";
const IDENTITY_VERSION_BYTES: [u8; 2] = 1_u16.to_be_bytes();
const HASH_ALGORITHM: [u8; 1] = [1];

pub(super) struct ChunkHasher {
    state: blake3::Hasher,
}

impl ChunkHasher {
    pub(super) fn new() -> Self {
        let mut state = blake3::Hasher::new();
        state.update(DATA_MAGIC);
        state.update(&IDENTITY_VERSION_BYTES);
        state.update(&HASH_ALGORITHM);
        Self { state }
    }

    pub(super) fn update(&mut self, bytes: &[u8]) {
        self.state.update(bytes);
    }

    pub(super) fn finish(mut self, length: ChunkLength) -> ChunkId {
        self.state.update(&length.get().to_be_bytes());
        let digest = *self.state.finalize().as_bytes();
        ChunkId::from_validated_parts(length, digest)
    }
}
