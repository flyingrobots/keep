//! Incremental physical segment digest construction for staged writes.

use blake3::Hasher;

use super::SegmentDigest;
use super::segment_seal_hash::{ALGORITHM, DIGEST_DOMAIN, VERSION};

pub(super) struct SegmentDigestBuilder {
    hasher: Hasher,
}

impl SegmentDigestBuilder {
    pub(super) fn new() -> Self {
        let mut hasher = Hasher::new();
        hasher.update(DIGEST_DOMAIN);
        hasher.update(&VERSION.to_be_bytes());
        hasher.update(&[ALGORITHM]);
        Self { hasher }
    }

    pub(super) fn update(&mut self, bytes: &[u8]) {
        self.hasher.update(bytes);
    }

    pub(super) fn finish(&self, seal_prefix: &[u8], input_length: u64) -> SegmentDigest {
        let mut hasher = self.hasher.clone();
        hasher.update(seal_prefix);
        hasher.update(&input_length.to_be_bytes());
        SegmentDigest::from_validated(*hasher.finalize().as_bytes())
    }
}
