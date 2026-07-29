//! Canonical catalog-entry state derived from one admitted segment record.

use super::segment_record_cursor::LocatedRecord;
use super::segment_record_identity_encoding;
use super::segment_record_kind::SegmentRecordKind;
use super::{
    SegmentDigest, SegmentRecordChecksum, SegmentRecordIdentity, SegmentRecordLength,
    SegmentRecordPayloadLength,
};

pub(super) const ENCODED_LENGTH: usize = 160;

pub(super) struct CatalogEncodingEntry {
    identity: SegmentRecordIdentity,
    segment_digest: SegmentDigest,
    record_offset: u64,
    record_length: SegmentRecordLength,
    payload_length: SegmentRecordPayloadLength,
    checksum: SegmentRecordChecksum,
}

impl CatalogEncodingEntry {
    pub(super) const fn from_located(
        segment_digest: SegmentDigest,
        located: &LocatedRecord<'_>,
    ) -> Self {
        let record = located.record;
        let header = record.header();
        Self {
            identity: record.identity(),
            segment_digest,
            record_offset: located.offset,
            record_length: header.record_length(),
            payload_length: header.payload_length(),
            checksum: record.checksum(),
        }
    }

    pub(super) const fn identity(&self) -> SegmentRecordIdentity {
        self.identity
    }

    pub(super) const fn encode(&self) -> [u8; ENCODED_LENGTH] {
        let mut encoded = [0_u8; ENCODED_LENGTH];
        let kind = SegmentRecordKind::from_identity(self.identity);
        let (kind_field, remaining) = encoded.split_at_mut(1);
        kind_field.copy_from_slice(&[kind.code()]);
        let (_flags, remaining) = remaining.split_at_mut(1);
        let (identity_length, remaining) = remaining.split_at_mut(2);
        identity_length.copy_from_slice(&kind.identity_length().to_be_bytes());
        let (identity, remaining) = remaining.split_at_mut(60);
        identity.copy_from_slice(&segment_record_identity_encoding::encode(self.identity));
        let (segment_digest, remaining) = remaining.split_at_mut(32);
        segment_digest.copy_from_slice(self.segment_digest.as_bytes());
        let (record_offset, remaining) = remaining.split_at_mut(8);
        record_offset.copy_from_slice(&self.record_offset.to_be_bytes());
        let (record_length, remaining) = remaining.split_at_mut(8);
        record_length.copy_from_slice(&self.record_length.get().to_be_bytes());
        let (payload_length, remaining) = remaining.split_at_mut(8);
        payload_length.copy_from_slice(&self.payload_length.get().to_be_bytes());
        let (checksum, _reserved) = remaining.split_at_mut(32);
        checksum.copy_from_slice(self.checksum.as_bytes());
        encoded
    }
}
