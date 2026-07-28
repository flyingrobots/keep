//! Canonical segment-record-header emitter.

use super::SegmentRecordIdentity;
use super::segment_record_header::{
    CHECKSUM_ALGORITHM, ENCODED_LENGTH, FLAGS, HEADER_LENGTH, IDENTITY_ALGORITHM, IDENTITY_VERSION,
    MAGIC, RECORD_VERSION, SegmentRecordHeader,
};
use super::segment_record_kind::SegmentRecordKind;

pub(super) const fn encode(header: SegmentRecordHeader) -> [u8; ENCODED_LENGTH] {
    let mut encoded = [0_u8; ENCODED_LENGTH];
    let identity = header.identity();
    let kind_coordinate = SegmentRecordKind::from_identity(identity);
    let (magic, remaining) = encoded.split_at_mut(16);
    magic.copy_from_slice(&MAGIC);
    let (version, remaining) = remaining.split_at_mut(2);
    version.copy_from_slice(&RECORD_VERSION.to_be_bytes());
    let (kind, remaining) = remaining.split_at_mut(1);
    kind.copy_from_slice(&[kind_coordinate.code()]);
    let (flags, remaining) = remaining.split_at_mut(1);
    flags.copy_from_slice(&[FLAGS]);
    let (header_length, remaining) = remaining.split_at_mut(2);
    header_length.copy_from_slice(&HEADER_LENGTH.to_be_bytes());
    let (identity_length, remaining) = remaining.split_at_mut(2);
    identity_length.copy_from_slice(&kind_coordinate.identity_length().to_be_bytes());
    let (payload_length, remaining) = remaining.split_at_mut(8);
    payload_length.copy_from_slice(&header.payload_length().get().to_be_bytes());
    let (record_length, remaining) = remaining.split_at_mut(8);
    record_length.copy_from_slice(&header.record_length().get().to_be_bytes());
    let (checksum_algorithm, remaining) = remaining.split_at_mut(1);
    checksum_algorithm.copy_from_slice(&[CHECKSUM_ALGORITHM]);
    let (identity_version, remaining) = remaining.split_at_mut(2);
    identity_version.copy_from_slice(&IDENTITY_VERSION.to_be_bytes());
    let (identity_algorithm, remaining) = remaining.split_at_mut(1);
    identity_algorithm.copy_from_slice(&[IDENTITY_ALGORITHM]);
    let (_reserved_prefix, remaining) = remaining.split_at_mut(4);
    let (identity_slot, _reserved_suffix) = remaining.split_at_mut(60);
    encode_identity(identity, identity_slot);
    encoded
}

const fn encode_identity(identity: SegmentRecordIdentity, slot: &mut [u8]) {
    match identity {
        SegmentRecordIdentity::Chunk(id) => {
            let (length, remaining) = slot.split_at_mut(4);
            length.copy_from_slice(&id.length().get().to_be_bytes());
            let (digest, _unused) = remaining.split_at_mut(32);
            digest.copy_from_slice(id.digest());
        }
        SegmentRecordIdentity::Layout(id) => slot.copy_from_slice(&id.encode_binary()),
    }
}
