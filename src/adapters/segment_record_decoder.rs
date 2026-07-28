//! Complete segment-record framing and checksum decoder.

use super::{
    ChecksummedSegmentRecord, SegmentRecordChecksum, SegmentRecordDecodeError, SegmentRecordHeader,
};

const CHECKSUM_LENGTH: u64 = 32;

pub(super) fn decode(
    encoded: &[u8],
) -> Result<ChecksummedSegmentRecord<'_>, SegmentRecordDecodeError> {
    let (header_bytes, remainder) = split_header(encoded)?;
    let header = SegmentRecordHeader::decode(header_bytes)
        .map_err(|source| SegmentRecordDecodeError::Header { source })?;
    validate_complete_length(encoded.len(), header)?;
    let payload_length = payload_host_length(header)?;
    let (payload, checksum_bytes) = remainder
        .split_at_checked(payload_length)
        .ok_or_else(|| truncated_record(encoded.len(), header))?;
    let (observed_bytes, trailing) = checksum_bytes
        .split_first_chunk::<32>()
        .ok_or_else(|| truncated_record(encoded.len(), header))?;
    if !trailing.is_empty() {
        return Err(SegmentRecordDecodeError::TrailingData {
            expected: header.record_length().get(),
            observed: encoded.len(),
        });
    }
    let covered_length = covered_length(header)?;
    let expected = SegmentRecordChecksum::calculate(header, payload, covered_length);
    let observed = SegmentRecordChecksum::from_validated(*observed_bytes);
    if expected != observed {
        return Err(SegmentRecordDecodeError::ChecksumMismatch { expected, observed });
    }
    Ok(ChecksummedSegmentRecord::from_verified_parts(
        header, payload, observed,
    ))
}

fn split_header(encoded: &[u8]) -> Result<(&[u8], &[u8]), SegmentRecordDecodeError> {
    encoded
        .split_at_checked(SegmentRecordHeader::ENCODED_LENGTH)
        .ok_or(SegmentRecordDecodeError::TruncatedHeader {
            expected: SegmentRecordHeader::ENCODED_LENGTH,
            observed: encoded.len(),
        })
}

fn validate_complete_length(
    observed: usize,
    header: SegmentRecordHeader,
) -> Result<(), SegmentRecordDecodeError> {
    let expected_u64 = header.record_length().get();
    let expected = usize::try_from(expected_u64).map_err(|_source| {
        SegmentRecordDecodeError::RecordLengthHostWidth {
            observed: expected_u64,
        }
    })?;
    match observed.cmp(&expected) {
        std::cmp::Ordering::Less => Err(SegmentRecordDecodeError::TruncatedRecord {
            expected: expected_u64,
            observed,
        }),
        std::cmp::Ordering::Greater => Err(SegmentRecordDecodeError::TrailingData {
            expected: expected_u64,
            observed,
        }),
        std::cmp::Ordering::Equal => Ok(()),
    }
}

fn payload_host_length(header: SegmentRecordHeader) -> Result<usize, SegmentRecordDecodeError> {
    let observed = header.payload_length().get();
    usize::try_from(observed)
        .map_err(|_source| SegmentRecordDecodeError::PayloadLengthHostWidth { observed })
}

fn covered_length(header: SegmentRecordHeader) -> Result<u64, SegmentRecordDecodeError> {
    header
        .record_length()
        .get()
        .checked_sub(CHECKSUM_LENGTH)
        .ok_or_else(|| SegmentRecordDecodeError::RecordLengthArithmetic {
            observed: header.record_length().get(),
        })
}

const fn truncated_record(
    observed: usize,
    header: SegmentRecordHeader,
) -> SegmentRecordDecodeError {
    SegmentRecordDecodeError::TruncatedRecord {
        expected: header.record_length().get(),
        observed,
    }
}
