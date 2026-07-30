//! This module owns fixed-framing validation for incomplete recovery segments.

use super::recovery_fixed_field_prefix::observed_field;
use super::segment_record_header::{
    CHECKSUM_ALGORITHM, FLAGS, HEADER_LENGTH, IDENTITY_ALGORITHM, IDENTITY_VERSION,
    MAGIC as RECORD_MAGIC, RECORD_VERSION,
};
use super::{
    SegmentHeader, SegmentHeaderError, SegmentRecordHeaderError,
    segment_record_kind::SegmentRecordKind, segment_seal,
};

pub(super) fn segment_header(encoded: &[u8]) -> Result<(), SegmentHeaderError> {
    let completed = observed_field(encoded, 0, SegmentHeader::admitted().encode());
    SegmentHeader::decode(&completed).map(|_header| ())
}

pub(super) fn segment_tail(encoded: &[u8]) -> Result<(), SegmentRecordHeaderError> {
    if segment_seal::MAGIC.starts_with(encoded) {
        return Ok(());
    }
    record_header(encoded)
}

fn record_header(encoded: &[u8]) -> Result<(), SegmentRecordHeaderError> {
    let magic = observed_field(encoded, 0, RECORD_MAGIC);
    if magic != RECORD_MAGIC {
        return Err(SegmentRecordHeaderError::InvalidMagic {
            expected: RECORD_MAGIC,
            observed: magic,
        });
    }
    let version = u16::from_be_bytes(observed_field(encoded, 16, RECORD_VERSION.to_be_bytes()));
    if version != RECORD_VERSION {
        return Err(SegmentRecordHeaderError::UnsupportedVersion {
            expected: RECORD_VERSION,
            observed: version,
        });
    }
    let kind = encoded
        .get(18)
        .copied()
        .map(SegmentRecordKind::admit)
        .transpose()?;
    let flags = u8::from_be_bytes(observed_field(encoded, 19, [FLAGS]));
    if flags != FLAGS {
        return Err(SegmentRecordHeaderError::UnknownFlags {
            expected: FLAGS,
            observed: flags,
        });
    }
    let header_length =
        u16::from_be_bytes(observed_field(encoded, 20, HEADER_LENGTH.to_be_bytes()));
    if header_length != HEADER_LENGTH {
        return Err(SegmentRecordHeaderError::HeaderLength {
            expected: HEADER_LENGTH,
            observed: header_length,
        });
    }
    if let Some(kind) = kind {
        validate_identity_length(encoded, kind)?;
    }
    validate_coordinates(encoded)
}

fn validate_identity_length(
    encoded: &[u8],
    kind: SegmentRecordKind,
) -> Result<(), SegmentRecordHeaderError> {
    let expected = kind.identity_length();
    let observed = u16::from_be_bytes(observed_field(encoded, 22, expected.to_be_bytes()));
    if observed == expected {
        Ok(())
    } else {
        Err(SegmentRecordHeaderError::IdentityLength {
            record_kind: kind.code(),
            expected,
            observed,
        })
    }
}

fn validate_coordinates(encoded: &[u8]) -> Result<(), SegmentRecordHeaderError> {
    let checksum = u8::from_be_bytes(observed_field(encoded, 40, [CHECKSUM_ALGORITHM]));
    if checksum != CHECKSUM_ALGORITHM {
        return Err(SegmentRecordHeaderError::RecordChecksumAlgorithm {
            expected: CHECKSUM_ALGORITHM,
            observed: checksum,
        });
    }
    let version = u16::from_be_bytes(observed_field(encoded, 41, IDENTITY_VERSION.to_be_bytes()));
    if version != IDENTITY_VERSION {
        return Err(SegmentRecordHeaderError::IdentityVersion {
            expected: IDENTITY_VERSION,
            observed: version,
        });
    }
    let algorithm = u8::from_be_bytes(observed_field(encoded, 43, [IDENTITY_ALGORITHM]));
    if algorithm != IDENTITY_ALGORITHM {
        return Err(SegmentRecordHeaderError::IdentityAlgorithm {
            expected: IDENTITY_ALGORITHM,
            observed: algorithm,
        });
    }
    validate_reserved(encoded, 44)?;
    validate_reserved(encoded, 108)
}

fn validate_reserved(encoded: &[u8], offset: u16) -> Result<(), SegmentRecordHeaderError> {
    let expected = [0_u8; 4];
    let observed = observed_field(encoded, usize::from(offset), expected);
    if observed == expected {
        Ok(())
    } else {
        Err(SegmentRecordHeaderError::ReservedBytes {
            offset,
            expected,
            observed,
        })
    }
}
