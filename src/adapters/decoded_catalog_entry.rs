//! Internal semantic catalog entry with unadmitted physical coordinates.

use super::{SegmentDigest, SegmentRecordChecksum, SegmentRecordIdentity, SegmentRecordLength};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct DecodedCatalogEntry {
    identity: SegmentRecordIdentity,
    segment_digest: SegmentDigest,
    record_offset: u64,
    record_length: SegmentRecordLength,
    checksum: SegmentRecordChecksum,
}

impl DecodedCatalogEntry {
    pub(super) const fn new(
        identity: SegmentRecordIdentity,
        segment_digest: SegmentDigest,
        record_offset: u64,
        record_length: SegmentRecordLength,
        checksum: SegmentRecordChecksum,
    ) -> Self {
        Self {
            identity,
            segment_digest,
            record_offset,
            record_length,
            checksum,
        }
    }

    pub(super) const fn identity(self) -> SegmentRecordIdentity {
        self.identity
    }

    pub(super) const fn segment_digest(self) -> SegmentDigest {
        self.segment_digest
    }

    pub(super) const fn record_offset(self) -> u64 {
        self.record_offset
    }

    pub(super) const fn record_length(self) -> SegmentRecordLength {
        self.record_length
    }

    pub(super) const fn checksum(self) -> SegmentRecordChecksum {
        self.checksum
    }
}
