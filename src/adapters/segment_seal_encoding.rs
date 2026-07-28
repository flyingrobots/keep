//! Canonical segment-seal emitter.

use super::segment_seal::{
    ALGORITHM, ENCODED_LENGTH, FLAGS, MAGIC, SEAL_LENGTH, SegmentSeal, VERSION,
};

pub(super) const fn encode(seal: SegmentSeal) -> [u8; ENCODED_LENGTH] {
    let mut encoded = [0_u8; ENCODED_LENGTH];
    let (magic, remaining) = encoded.split_at_mut(16);
    magic.copy_from_slice(&MAGIC);
    let (version, remaining) = remaining.split_at_mut(2);
    version.copy_from_slice(&VERSION.to_be_bytes());
    let (flags, remaining) = remaining.split_at_mut(2);
    flags.copy_from_slice(&FLAGS.to_be_bytes());
    let (length, remaining) = remaining.split_at_mut(2);
    length.copy_from_slice(&SEAL_LENGTH.to_be_bytes());
    let (_reserved_u16, remaining) = remaining.split_at_mut(2);
    let (record_count, remaining) = remaining.split_at_mut(4);
    record_count.copy_from_slice(&seal.record_count().to_be_bytes());
    let (_reserved_u32, remaining) = remaining.split_at_mut(4);
    let (bytes_before_seal, remaining) = remaining.split_at_mut(8);
    bytes_before_seal.copy_from_slice(&seal.bytes_before_seal().to_be_bytes());
    let (segment_length, remaining) = remaining.split_at_mut(8);
    segment_length.copy_from_slice(&seal.segment_length().to_be_bytes());
    let (record_bytes, remaining) = remaining.split_at_mut(8);
    record_bytes.copy_from_slice(&seal.record_bytes().to_be_bytes());
    let (checksum_algorithm, remaining) = remaining.split_at_mut(1);
    checksum_algorithm.copy_from_slice(&[ALGORITHM]);
    let (digest_algorithm, remaining) = remaining.split_at_mut(1);
    digest_algorithm.copy_from_slice(&[ALGORITHM]);
    let (_reserved, remaining) = remaining.split_at_mut(6);
    let (digest, checksum) = remaining.split_at_mut(32);
    digest.copy_from_slice(seal.digest().as_bytes());
    checksum.copy_from_slice(&seal.checksum());
    encoded
}
