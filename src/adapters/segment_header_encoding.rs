//! Canonical version-1 segment-header emission.

use super::segment_header::{
    BLAKE3_256, ENCODED_LENGTH, FLAGS, FORMAT_VERSION, HEADER_LENGTH, MAGIC, MAXIMUM_RECORD_COUNT,
    MAXIMUM_RECORD_PAYLOAD_LENGTH, MAXIMUM_SEGMENT_LENGTH, RECORD_HEADER_LENGTH, SEAL_LENGTH,
};

pub(super) const fn canonical_bytes() -> [u8; ENCODED_LENGTH] {
    let mut encoded = [0_u8; ENCODED_LENGTH];
    let (magic, remaining) = encoded.split_at_mut(16);
    magic.copy_from_slice(&MAGIC);
    let (version, remaining) = remaining.split_at_mut(2);
    version.copy_from_slice(&FORMAT_VERSION.to_be_bytes());
    let (flags, remaining) = remaining.split_at_mut(2);
    flags.copy_from_slice(&FLAGS.to_be_bytes());
    let (header_length, remaining) = remaining.split_at_mut(2);
    header_length.copy_from_slice(&HEADER_LENGTH.to_be_bytes());
    let (record_header_length, remaining) = remaining.split_at_mut(2);
    record_header_length.copy_from_slice(&RECORD_HEADER_LENGTH.to_be_bytes());
    let (seal_length, remaining) = remaining.split_at_mut(2);
    seal_length.copy_from_slice(&SEAL_LENGTH.to_be_bytes());
    let (_reserved, remaining) = remaining.split_at_mut(2);
    let (payload_bound, remaining) = remaining.split_at_mut(8);
    payload_bound.copy_from_slice(&MAXIMUM_RECORD_PAYLOAD_LENGTH.to_be_bytes());
    let (segment_bound, remaining) = remaining.split_at_mut(8);
    segment_bound.copy_from_slice(&MAXIMUM_SEGMENT_LENGTH.to_be_bytes());
    let (record_bound, remaining) = remaining.split_at_mut(4);
    record_bound.copy_from_slice(&MAXIMUM_RECORD_COUNT.to_be_bytes());
    let (record_algorithm, remaining) = remaining.split_at_mut(1);
    record_algorithm.copy_from_slice(&[BLAKE3_256]);
    let (segment_algorithm, _reserved) = remaining.split_at_mut(1);
    segment_algorithm.copy_from_slice(&[BLAKE3_256]);
    encoded
}
