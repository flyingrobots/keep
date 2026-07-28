//! Fixed-width segment-record-header field decoder.

use super::SegmentRecordHeaderError;
use super::segment_record_header::{ENCODED_LENGTH, SegmentRecordHeader};
use super::segment_record_header_admission;

pub(super) struct DecodedFields {
    pub(super) magic: [u8; 16],
    pub(super) record_version: u16,
    pub(super) record_kind: u8,
    pub(super) flags: u8,
    pub(super) header_length: u16,
    pub(super) identity_length: u16,
    pub(super) payload_length: u64,
    pub(super) record_length: u64,
    pub(super) checksum_algorithm: u8,
    pub(super) identity_version: u16,
    pub(super) identity_algorithm: u8,
    pub(super) reserved_prefix: [u8; 4],
    pub(super) identity: [u8; 60],
    pub(super) reserved_suffix: [u8; 4],
}

pub(super) fn decode(encoded: &[u8]) -> Result<SegmentRecordHeader, SegmentRecordHeaderError> {
    if encoded.len() != ENCODED_LENGTH {
        return Err(wrong_length(encoded.len()));
    }
    let fields = decode_fields(encoded)?;
    segment_record_header_admission::admit(&fields)
}

fn decode_fields(encoded: &[u8]) -> Result<DecodedFields, SegmentRecordHeaderError> {
    let observed = encoded.len();
    let (magic, remainder) = read_array::<16>(encoded, observed)?;
    let (record_version, remainder) = read_u16(remainder, observed)?;
    let (record_kind, remainder) = read_u8(remainder, observed)?;
    let (flags, remainder) = read_u8(remainder, observed)?;
    let (header_length, remainder) = read_u16(remainder, observed)?;
    let (identity_length, remainder) = read_u16(remainder, observed)?;
    let (payload_length, remainder) = read_u64(remainder, observed)?;
    let (record_length, remainder) = read_u64(remainder, observed)?;
    let (checksum_algorithm, remainder) = read_u8(remainder, observed)?;
    let (identity_version, remainder) = read_u16(remainder, observed)?;
    let (identity_algorithm, remainder) = read_u8(remainder, observed)?;
    let (reserved_prefix, remainder) = read_array::<4>(remainder, observed)?;
    let (identity, remainder) = read_array::<60>(remainder, observed)?;
    let (reserved_suffix, trailing) = read_array::<4>(remainder, observed)?;
    if !trailing.is_empty() {
        return Err(wrong_length(observed));
    }
    Ok(DecodedFields {
        magic,
        record_version,
        record_kind,
        flags,
        header_length,
        identity_length,
        payload_length,
        record_length,
        checksum_algorithm,
        identity_version,
        identity_algorithm,
        reserved_prefix,
        identity,
        reserved_suffix,
    })
}

const fn read_u8(bytes: &[u8], observed: usize) -> Result<(u8, &[u8]), SegmentRecordHeaderError> {
    let Some((value, remainder)) = bytes.split_first() else {
        return Err(wrong_length(observed));
    };
    Ok((*value, remainder))
}

fn read_u16(bytes: &[u8], observed: usize) -> Result<(u16, &[u8]), SegmentRecordHeaderError> {
    let (value, remainder) = read_array::<2>(bytes, observed)?;
    Ok((u16::from_be_bytes(value), remainder))
}

fn read_u64(bytes: &[u8], observed: usize) -> Result<(u64, &[u8]), SegmentRecordHeaderError> {
    let (value, remainder) = read_array::<8>(bytes, observed)?;
    Ok((u64::from_be_bytes(value), remainder))
}

const fn read_array<const N: usize>(
    bytes: &[u8],
    observed: usize,
) -> Result<([u8; N], &[u8]), SegmentRecordHeaderError> {
    let Some((value, remainder)) = bytes.split_first_chunk::<N>() else {
        return Err(wrong_length(observed));
    };
    Ok((*value, remainder))
}

const fn wrong_length(observed: usize) -> SegmentRecordHeaderError {
    SegmentRecordHeaderError::WrongLength {
        expected: ENCODED_LENGTH,
        observed,
    }
}
