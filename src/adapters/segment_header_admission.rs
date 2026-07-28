//! Version-1 segment-header field admission.

use super::SegmentHeaderError;
use super::segment_header::{
    BLAKE3_256, FLAGS, FORMAT_VERSION, HEADER_LENGTH, MAGIC, MAXIMUM_RECORD_COUNT,
    MAXIMUM_RECORD_PAYLOAD_LENGTH, MAXIMUM_SEGMENT_LENGTH, RECORD_HEADER_LENGTH, SEAL_LENGTH,
    SegmentHeader,
};
use super::segment_header_decoder::DecodedFields;

pub(super) fn admit(fields: &DecodedFields) -> Result<SegmentHeader, SegmentHeaderError> {
    validate_prefix(fields)?;
    validate_bounds(fields)?;
    validate_algorithms(fields)?;
    Ok(SegmentHeader::admitted())
}

fn validate_prefix(fields: &DecodedFields) -> Result<(), SegmentHeaderError> {
    require_bytes(fields.magic, MAGIC)?;
    require_u16_version(fields.version)?;
    require_u16(fields.flags, FLAGS, |expected, observed| {
        SegmentHeaderError::UnknownFlags { expected, observed }
    })?;
    require_u16(fields.header_length, HEADER_LENGTH, |expected, observed| {
        SegmentHeaderError::HeaderLength { expected, observed }
    })?;
    require_u16(
        fields.record_header_length,
        RECORD_HEADER_LENGTH,
        |expected, observed| SegmentHeaderError::RecordHeaderLength { expected, observed },
    )?;
    require_u16(fields.seal_length, SEAL_LENGTH, |expected, observed| {
        SegmentHeaderError::SealLength { expected, observed }
    })?;
    if fields.reserved != 0 {
        return Err(SegmentHeaderError::ReservedU16 {
            offset: 26,
            expected: 0,
            observed: fields.reserved,
        });
    }
    Ok(())
}

fn validate_bounds(fields: &DecodedFields) -> Result<(), SegmentHeaderError> {
    require_u64(
        fields.maximum_record_payload_length,
        MAXIMUM_RECORD_PAYLOAD_LENGTH,
        |expected, observed| SegmentHeaderError::MaximumRecordPayloadLength { expected, observed },
    )?;
    require_u64(
        fields.maximum_segment_length,
        MAXIMUM_SEGMENT_LENGTH,
        |expected, observed| SegmentHeaderError::MaximumSegmentLength { expected, observed },
    )?;
    if fields.maximum_record_count != MAXIMUM_RECORD_COUNT {
        return Err(SegmentHeaderError::MaximumRecordCount {
            expected: MAXIMUM_RECORD_COUNT,
            observed: fields.maximum_record_count,
        });
    }
    Ok(())
}

fn validate_algorithms(fields: &DecodedFields) -> Result<(), SegmentHeaderError> {
    require_u8(
        fields.record_checksum_algorithm,
        BLAKE3_256,
        |expected, observed| SegmentHeaderError::RecordChecksumAlgorithm { expected, observed },
    )?;
    require_u8(
        fields.segment_digest_algorithm,
        BLAKE3_256,
        |expected, observed| SegmentHeaderError::SegmentDigestAlgorithm { expected, observed },
    )?;
    if fields.trailing_reserved != [0; 14] {
        return Err(SegmentHeaderError::ReservedBytes {
            offset: 50,
            expected: [0; 14],
            observed: fields.trailing_reserved,
        });
    }
    Ok(())
}

fn require_bytes(observed: [u8; 16], expected: [u8; 16]) -> Result<(), SegmentHeaderError> {
    if observed == expected {
        return Ok(());
    }
    Err(SegmentHeaderError::InvalidMagic { expected, observed })
}

const fn require_u16_version(observed: u16) -> Result<(), SegmentHeaderError> {
    if observed == FORMAT_VERSION {
        return Ok(());
    }
    Err(SegmentHeaderError::UnsupportedVersion {
        expected: FORMAT_VERSION,
        observed,
    })
}

fn require_u16(
    observed: u16,
    expected: u16,
    error: fn(u16, u16) -> SegmentHeaderError,
) -> Result<(), SegmentHeaderError> {
    if observed == expected {
        return Ok(());
    }
    Err(error(expected, observed))
}

fn require_u64(
    observed: u64,
    expected: u64,
    error: fn(u64, u64) -> SegmentHeaderError,
) -> Result<(), SegmentHeaderError> {
    if observed == expected {
        return Ok(());
    }
    Err(error(expected, observed))
}

fn require_u8(
    observed: u8,
    expected: u8,
    error: fn(u8, u8) -> SegmentHeaderError,
) -> Result<(), SegmentHeaderError> {
    if observed == expected {
        return Ok(());
    }
    Err(error(expected, observed))
}
