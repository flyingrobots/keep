//! This module owns one canonical retention-closure member entry.

use crate::{AdmittedSegmentRecord, SegmentRecordIdentity};

const ENTRY_LENGTH: usize = 96;
const CHUNK_KIND: u8 = 1;
const LAYOUT_KIND: u8 = 2;

pub(super) struct ClosureMember([u8; ENTRY_LENGTH]);

impl ClosureMember {
    pub(super) const fn new(
        identity: SegmentRecordIdentity,
        record: AdmittedSegmentRecord<'_>,
    ) -> Self {
        let mut encoded = [0_u8; ENTRY_LENGTH];
        let (kind_slot, remainder) = encoded.split_at_mut(1);
        kind_slot.copy_from_slice(&[kind(identity)]);
        let (_reserved, remainder) = remainder.split_at_mut(3);
        let (identity_slot, checksum_slot) = remainder.split_at_mut(60);
        identity_slot.copy_from_slice(&crate::adapters::segment_record_identity_encoding::encode(
            identity,
        ));
        checksum_slot.copy_from_slice(record.checksum().as_bytes());
        Self(encoded)
    }

    pub(super) const fn as_bytes(&self) -> &[u8; ENTRY_LENGTH] {
        &self.0
    }
}

const fn kind(identity: SegmentRecordIdentity) -> u8 {
    match identity {
        SegmentRecordIdentity::Chunk(_) => CHUNK_KIND,
        SegmentRecordIdentity::Layout(_) => LAYOUT_KIND,
    }
}
