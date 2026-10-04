//! Structurally and logically admitted borrowed immutable segment.

use super::segment_record_cursor::SegmentRecordCursor;
use super::{
    SegmentDigest, SegmentReadError, SegmentReadPolicy, SegmentRecords, SegmentSeal, segment_reader,
};

/// A borrowed complete segment admitted through its header, records, and seal.
///
/// Admission verifies the physical segment digest and seal checksum, every
/// record checksum and logical identity, exact record count and terminal
/// framing, and duplicate logical identities. Duplicate detection temporarily
/// allocates one bounded identity coordinate per declared record. This type
/// performs no I/O and makes no durability, publication, or retention claim.
#[must_use]
#[derive(Debug)]
pub struct AdmittedSegment<'a> {
    encoded: &'a [u8],
    records: &'a [u8],
    seal: SegmentSeal,
    policy: SegmentReadPolicy,
}

impl<'a> AdmittedSegment<'a> {
    /// Admits one complete immutable segment under explicit resource bounds.
    ///
    /// # Errors
    ///
    /// Returns [`SegmentReadError`] at the first failed outer-integrity,
    /// configured-resource, record-framing, content-identity, trailing-byte,
    /// allocation-reservation, or duplicate-identity law.
    pub fn decode(encoded: &'a [u8], policy: SegmentReadPolicy) -> Result<Self, SegmentReadError> {
        segment_reader::decode(encoded, policy)
    }

    /// Returns the exact immutable encoded segment bytes.
    #[must_use]
    pub const fn encoded(&self) -> &'a [u8] {
        self.encoded
    }

    /// Returns the exact declared and verified record count.
    #[must_use]
    pub const fn record_count(&self) -> u32 {
        self.seal.record_count()
    }

    /// Returns the verified physical immutable-segment digest.
    #[must_use]
    pub const fn digest(&self) -> SegmentDigest {
        self.seal.digest()
    }

    pub(super) const fn segment_length(&self) -> u64 {
        self.seal.segment_length()
    }

    /// Returns a revalidating iterator over records in physical order.
    #[must_use]
    pub const fn records(&self) -> SegmentRecords<'a> {
        SegmentRecords::new(self.records, self.seal.record_count(), self.policy)
    }

    pub(super) const fn record_cursor(&self) -> SegmentRecordCursor<'a> {
        SegmentRecordCursor::new(self.records, self.seal.record_count(), self.policy)
    }

    pub(super) const fn admitted(
        encoded: &'a [u8],
        records: &'a [u8],
        seal: SegmentSeal,
        policy: SegmentReadPolicy,
    ) -> Self {
        Self {
            encoded,
            records,
            seal,
            policy,
        }
    }
}
