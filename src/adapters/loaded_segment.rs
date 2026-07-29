//! Owned bytes for one catalog-selected segment.

use super::SegmentDigest;

#[derive(Debug)]
pub(super) struct LoadedSegment {
    digest: SegmentDigest,
    encoded: Vec<u8>,
}

impl LoadedSegment {
    pub(super) const fn new(digest: SegmentDigest, encoded: Vec<u8>) -> Self {
        Self { digest, encoded }
    }

    pub(super) const fn digest(&self) -> SegmentDigest {
        self.digest
    }

    pub(super) fn encoded(&self) -> &[u8] {
        &self.encoded
    }
}
