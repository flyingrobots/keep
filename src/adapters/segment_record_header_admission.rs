//! Segment-record-header semantic admission.

use super::segment_record_header::{
    CHECKSUM_ALGORITHM, CHECKSUM_LENGTH, FLAGS, HEADER_LENGTH, IDENTITY_ALGORITHM,
    IDENTITY_VERSION, MAGIC, RECORD_VERSION, SegmentRecordHeader,
};
use super::segment_record_header_decoder::DecodedFields;
use super::segment_record_identity_admission;
use super::segment_record_kind::SegmentRecordKind;
use super::{
    SegmentRecordHeaderError, SegmentRecordIdentity, SegmentRecordLength,
    SegmentRecordPayloadLength,
};

pub(super) fn admit(
    fields: &DecodedFields,
) -> Result<SegmentRecordHeader, SegmentRecordHeaderError> {
    validate_prefix(fields)?;
    let kind = SegmentRecordKind::admit(fields.record_kind)?;
    validate_framing(fields, kind)?;
    validate_coordinates(fields)?;
    let identity = segment_record_identity_admission::admit(fields, kind)?;
    validate_reserved(fields.reserved_suffix, 108)?;
    from_identity(identity)
}

pub(super) fn from_identity(
    identity: SegmentRecordIdentity,
) -> Result<SegmentRecordHeader, SegmentRecordHeaderError> {
    let payload = identity.payload_length();
    let kind = SegmentRecordKind::from_identity(identity);
    validate_payload_length(payload, kind)?;
    let record_length = calculate_record_length(payload)?;
    Ok(SegmentRecordHeader::admitted(
        identity,
        SegmentRecordPayloadLength::from_validated(payload),
        SegmentRecordLength::from_validated(record_length),
    ))
}

fn validate_prefix(fields: &DecodedFields) -> Result<(), SegmentRecordHeaderError> {
    if fields.magic != MAGIC {
        return Err(SegmentRecordHeaderError::InvalidMagic {
            expected: MAGIC,
            observed: fields.magic,
        });
    }
    if fields.record_version != RECORD_VERSION {
        return Err(SegmentRecordHeaderError::UnsupportedVersion {
            expected: RECORD_VERSION,
            observed: fields.record_version,
        });
    }
    Ok(())
}

fn validate_framing(
    fields: &DecodedFields,
    kind: SegmentRecordKind,
) -> Result<(), SegmentRecordHeaderError> {
    require_u8(fields.flags, FLAGS, |expected, observed| {
        SegmentRecordHeaderError::UnknownFlags { expected, observed }
    })?;
    require_u16(fields.header_length, HEADER_LENGTH, |expected, observed| {
        SegmentRecordHeaderError::HeaderLength { expected, observed }
    })?;
    let expected_identity_length = kind.identity_length();
    if fields.identity_length != expected_identity_length {
        return Err(SegmentRecordHeaderError::IdentityLength {
            record_kind: kind.code(),
            expected: expected_identity_length,
            observed: fields.identity_length,
        });
    }
    validate_payload_length(fields.payload_length, kind)?;
    let expected_record_length = calculate_record_length(fields.payload_length)?;
    if fields.record_length != expected_record_length {
        return Err(SegmentRecordHeaderError::RecordLength {
            expected: expected_record_length,
            observed: fields.record_length,
        });
    }
    Ok(())
}

fn validate_coordinates(fields: &DecodedFields) -> Result<(), SegmentRecordHeaderError> {
    require_u8(
        fields.checksum_algorithm,
        CHECKSUM_ALGORITHM,
        |expected, observed| SegmentRecordHeaderError::RecordChecksumAlgorithm {
            expected,
            observed,
        },
    )?;
    require_u16(
        fields.identity_version,
        IDENTITY_VERSION,
        |expected, observed| SegmentRecordHeaderError::IdentityVersion { expected, observed },
    )?;
    require_u8(
        fields.identity_algorithm,
        IDENTITY_ALGORITHM,
        |expected, observed| SegmentRecordHeaderError::IdentityAlgorithm { expected, observed },
    )?;
    validate_reserved(fields.reserved_prefix, 44)
}

fn validate_payload_length(
    observed: u64,
    kind: SegmentRecordKind,
) -> Result<(), SegmentRecordHeaderError> {
    let (minimum, maximum) = kind.payload_bounds();
    if (minimum..=maximum).contains(&observed) {
        return Ok(());
    }
    Err(SegmentRecordHeaderError::PayloadLengthOutOfBounds {
        record_kind: kind.code(),
        minimum,
        maximum,
        observed,
    })
}

fn calculate_record_length(payload_length: u64) -> Result<u64, SegmentRecordHeaderError> {
    u64::from(HEADER_LENGTH)
        .checked_add(payload_length)
        .and_then(|length| length.checked_add(CHECKSUM_LENGTH))
        .ok_or(SegmentRecordHeaderError::RecordLengthArithmetic { payload_length })
}

fn validate_reserved(observed: [u8; 4], offset: u16) -> Result<(), SegmentRecordHeaderError> {
    let expected = [0_u8; 4];
    if observed == expected {
        return Ok(());
    }
    Err(SegmentRecordHeaderError::ReservedBytes {
        offset,
        expected,
        observed,
    })
}

fn require_u8(
    observed: u8,
    expected: u8,
    error: fn(u8, u8) -> SegmentRecordHeaderError,
) -> Result<(), SegmentRecordHeaderError> {
    if observed == expected {
        return Ok(());
    }
    Err(error(expected, observed))
}

fn require_u16(
    observed: u16,
    expected: u16,
    error: fn(u16, u16) -> SegmentRecordHeaderError,
) -> Result<(), SegmentRecordHeaderError> {
    if observed == expected {
        return Ok(());
    }
    Err(error(expected, observed))
}
