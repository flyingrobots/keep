//! Fixed-width segment-header field decoding.

use super::SegmentHeaderError;
use super::segment_header::{ENCODED_LENGTH, SegmentHeader};
use super::segment_header_admission;

pub(super) struct DecodedFields {
    pub(super) magic: [u8; 16],
    pub(super) version: u16,
    pub(super) flags: u16,
    pub(super) header_length: u16,
    pub(super) record_header_length: u16,
    pub(super) seal_length: u16,
    pub(super) reserved: u16,
    pub(super) maximum_record_payload_length: u64,
    pub(super) maximum_segment_length: u64,
    pub(super) maximum_record_count: u32,
    pub(super) record_checksum_algorithm: u8,
    pub(super) segment_digest_algorithm: u8,
    pub(super) trailing_reserved: [u8; 14],
}

pub(super) fn decode(encoded: &[u8]) -> Result<SegmentHeader, SegmentHeaderError> {
    if encoded.len() != ENCODED_LENGTH {
        return Err(wrong_length(encoded.len()));
    }
    let fields = decode_fields(encoded)?;
    segment_header_admission::admit(&fields)
}

fn decode_fields(encoded: &[u8]) -> Result<DecodedFields, SegmentHeaderError> {
    let observed = encoded.len();
    let (magic, remaining) = read_array::<16>(encoded, observed)?;
    let (version, remaining) = read_u16(remaining, observed)?;
    let (flags, remaining) = read_u16(remaining, observed)?;
    let (header_length, remaining) = read_u16(remaining, observed)?;
    let (record_header_length, remaining) = read_u16(remaining, observed)?;
    let (seal_length, remaining) = read_u16(remaining, observed)?;
    let (reserved, remaining) = read_u16(remaining, observed)?;
    let (maximum_record_payload_length, remaining) = read_u64(remaining, observed)?;
    let (maximum_segment_length, remaining) = read_u64(remaining, observed)?;
    let (maximum_record_count, remaining) = read_u32(remaining, observed)?;
    let Some((&record_checksum_algorithm, remaining)) = remaining.split_first() else {
        return Err(wrong_length(observed));
    };
    let Some((&segment_digest_algorithm, remaining)) = remaining.split_first() else {
        return Err(wrong_length(observed));
    };
    let (trailing_reserved, trailing) = read_array::<14>(remaining, observed)?;
    if !trailing.is_empty() {
        return Err(wrong_length(observed));
    }
    Ok(DecodedFields {
        magic,
        version,
        flags,
        header_length,
        record_header_length,
        seal_length,
        reserved,
        maximum_record_payload_length,
        maximum_segment_length,
        maximum_record_count,
        record_checksum_algorithm,
        segment_digest_algorithm,
        trailing_reserved,
    })
}

fn read_u16(bytes: &[u8], observed: usize) -> Result<(u16, &[u8]), SegmentHeaderError> {
    let (value, remaining) = read_array::<2>(bytes, observed)?;
    Ok((u16::from_be_bytes(value), remaining))
}

fn read_u32(bytes: &[u8], observed: usize) -> Result<(u32, &[u8]), SegmentHeaderError> {
    let (value, remaining) = read_array::<4>(bytes, observed)?;
    Ok((u32::from_be_bytes(value), remaining))
}

fn read_u64(bytes: &[u8], observed: usize) -> Result<(u64, &[u8]), SegmentHeaderError> {
    let (value, remaining) = read_array::<8>(bytes, observed)?;
    Ok((u64::from_be_bytes(value), remaining))
}

fn read_array<const N: usize>(
    bytes: &[u8],
    observed: usize,
) -> Result<([u8; N], &[u8]), SegmentHeaderError> {
    let Some((value, remaining)) = bytes.split_first_chunk::<N>() else {
        return Err(wrong_length(observed));
    };
    Ok((*value, remaining))
}

const fn wrong_length(observed: usize) -> SegmentHeaderError {
    SegmentHeaderError::WrongLength {
        expected: ENCODED_LENGTH,
        observed,
    }
}
