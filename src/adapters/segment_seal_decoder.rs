//! Fixed-width segment-seal field decoder.

use super::SegmentSealError;
use super::segment_seal::{ENCODED_LENGTH, SegmentSeal};
use super::segment_seal_admission;

pub(super) struct DecodedSeal {
    pub(super) magic: [u8; 16],
    pub(super) version: u16,
    pub(super) flags: u16,
    pub(super) seal_length: u16,
    pub(super) reserved_u16: u16,
    pub(super) record_count: u32,
    pub(super) reserved_u32: u32,
    pub(super) bytes_before_seal: u64,
    pub(super) segment_length: u64,
    pub(super) record_bytes: u64,
    pub(super) checksum_algorithm: u8,
    pub(super) digest_algorithm: u8,
    pub(super) reserved: [u8; 6],
    pub(super) digest: [u8; 32],
    pub(super) checksum: [u8; 32],
}

pub(super) fn decode(prefix: &[u8], encoded: &[u8]) -> Result<SegmentSeal, SegmentSealError> {
    if encoded.len() != ENCODED_LENGTH {
        return Err(wrong_length(encoded.len()));
    }
    let fields = decode_fields(encoded)?;
    segment_seal_admission::admit(prefix, &fields)
}

pub(super) fn decode_fields(encoded: &[u8]) -> Result<DecodedSeal, SegmentSealError> {
    let observed = encoded.len();
    let (magic, remainder) = read_array::<16>(encoded, observed)?;
    let (version, remainder) = read_u16(remainder, observed)?;
    let (flags, remainder) = read_u16(remainder, observed)?;
    let (seal_length, remainder) = read_u16(remainder, observed)?;
    let (reserved_u16, remainder) = read_u16(remainder, observed)?;
    let (record_count, remainder) = read_u32(remainder, observed)?;
    let (reserved_u32, remainder) = read_u32(remainder, observed)?;
    let (bytes_before_seal, remainder) = read_u64(remainder, observed)?;
    let (segment_length, remainder) = read_u64(remainder, observed)?;
    let (record_bytes, remainder) = read_u64(remainder, observed)?;
    let (checksum_algorithm, remainder) = read_u8(remainder, observed)?;
    let (digest_algorithm, remainder) = read_u8(remainder, observed)?;
    let (reserved, remainder) = read_array::<6>(remainder, observed)?;
    let (digest, remainder) = read_array::<32>(remainder, observed)?;
    let (checksum, trailing) = read_array::<32>(remainder, observed)?;
    if !trailing.is_empty() {
        return Err(wrong_length(observed));
    }
    Ok(DecodedSeal {
        magic,
        version,
        flags,
        seal_length,
        reserved_u16,
        record_count,
        reserved_u32,
        bytes_before_seal,
        segment_length,
        record_bytes,
        checksum_algorithm,
        digest_algorithm,
        reserved,
        digest,
        checksum,
    })
}

const fn read_u8(bytes: &[u8], observed: usize) -> Result<(u8, &[u8]), SegmentSealError> {
    let Some((value, remainder)) = bytes.split_first() else {
        return Err(wrong_length(observed));
    };
    Ok((*value, remainder))
}

fn read_u16(bytes: &[u8], observed: usize) -> Result<(u16, &[u8]), SegmentSealError> {
    let (value, remainder) = read_array::<2>(bytes, observed)?;
    Ok((u16::from_be_bytes(value), remainder))
}

fn read_u32(bytes: &[u8], observed: usize) -> Result<(u32, &[u8]), SegmentSealError> {
    let (value, remainder) = read_array::<4>(bytes, observed)?;
    Ok((u32::from_be_bytes(value), remainder))
}

fn read_u64(bytes: &[u8], observed: usize) -> Result<(u64, &[u8]), SegmentSealError> {
    let (value, remainder) = read_array::<8>(bytes, observed)?;
    Ok((u64::from_be_bytes(value), remainder))
}

const fn read_array<const N: usize>(
    bytes: &[u8],
    observed: usize,
) -> Result<([u8; N], &[u8]), SegmentSealError> {
    let Some((value, remainder)) = bytes.split_first_chunk::<N>() else {
        return Err(wrong_length(observed));
    };
    Ok((*value, remainder))
}

const fn wrong_length(observed: usize) -> SegmentSealError {
    SegmentSealError::WrongLength {
        expected: ENCODED_LENGTH,
        observed,
    }
}
