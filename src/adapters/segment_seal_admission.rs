//! Segment-seal semantic and cryptographic admission.

use super::segment_header::{HEADER_LENGTH, MAXIMUM_RECORD_COUNT, MAXIMUM_SEGMENT_LENGTH};
use super::segment_seal::{
    ALGORITHM, FLAGS, MAGIC, SEAL_LENGTH, SegmentSeal, SegmentSealCoordinates, VERSION,
};
use super::segment_seal_decoder::DecodedSeal;
use super::{SegmentDigest, SegmentSealError, segment_seal_hash};

pub(super) fn admit(prefix: &[u8], fields: &DecodedSeal) -> Result<SegmentSeal, SegmentSealError> {
    validate_coordinates(fields)?;
    let prefix_length = prefix_length(prefix.len())?;
    validate_lengths(fields, prefix_length)?;
    validate_algorithms(fields)?;
    let canonical = from_prefix(prefix, fields.record_count)?;
    let observed_digest = SegmentDigest::from_validated(fields.digest);
    if observed_digest != canonical.digest() {
        return Err(SegmentSealError::SegmentDigestMismatch {
            expected: canonical.digest(),
            observed: observed_digest,
        });
    }
    if fields.checksum != canonical.checksum() {
        return Err(SegmentSealError::SealChecksumMismatch {
            expected: canonical.checksum(),
            observed: fields.checksum,
        });
    }
    Ok(canonical)
}

pub(super) fn from_prefix(
    prefix: &[u8],
    record_count: u32,
) -> Result<SegmentSeal, SegmentSealError> {
    if record_count > MAXIMUM_RECORD_COUNT {
        return Err(SegmentSealError::RecordCountOutOfBounds {
            maximum: MAXIMUM_RECORD_COUNT,
            observed: record_count,
        });
    }
    let bytes_before_seal = prefix_length(prefix.len())?;
    let (segment_length, record_bytes) = derived_lengths(bytes_before_seal)?;
    let coordinates = SegmentSealCoordinates::new(
        record_count,
        bytes_before_seal,
        segment_length,
        record_bytes,
    );
    let provisional = SegmentSeal::admitted(
        coordinates,
        SegmentDigest::from_validated([0_u8; 32]),
        [0_u8; 32],
    );
    let provisional_bytes = provisional.encode();
    let digest = segment_seal_hash::segment_digest(prefix, &provisional_bytes)?;
    let with_digest = SegmentSeal::admitted(coordinates, digest, [0_u8; 32]);
    let checksum = segment_seal_hash::seal_checksum(&with_digest.encode())?;
    Ok(SegmentSeal::admitted(coordinates, digest, checksum))
}

fn validate_coordinates(fields: &DecodedSeal) -> Result<(), SegmentSealError> {
    if fields.magic != MAGIC {
        return Err(SegmentSealError::InvalidMagic {
            expected: MAGIC,
            observed: fields.magic,
        });
    }
    if fields.version != VERSION {
        return Err(SegmentSealError::UnsupportedVersion {
            expected: VERSION,
            observed: fields.version,
        });
    }
    require_u16(fields.flags, FLAGS, |expected, observed| {
        SegmentSealError::UnknownFlags { expected, observed }
    })?;
    require_u16(fields.seal_length, SEAL_LENGTH, |expected, observed| {
        SegmentSealError::SealLength { expected, observed }
    })?;
    require_u16(fields.reserved_u16, 0, |expected, observed| {
        SegmentSealError::ReservedU16 { expected, observed }
    })?;
    if fields.record_count > MAXIMUM_RECORD_COUNT {
        return Err(SegmentSealError::RecordCountOutOfBounds {
            maximum: MAXIMUM_RECORD_COUNT,
            observed: fields.record_count,
        });
    }
    if fields.reserved_u32 != 0 {
        return Err(SegmentSealError::ReservedU32 {
            expected: 0,
            observed: fields.reserved_u32,
        });
    }
    Ok(())
}

fn validate_lengths(fields: &DecodedSeal, prefix_length: u64) -> Result<(), SegmentSealError> {
    if fields.bytes_before_seal != prefix_length {
        return Err(SegmentSealError::BytesBeforeSeal {
            expected: prefix_length,
            observed: fields.bytes_before_seal,
        });
    }
    let (segment_length, record_bytes) = derived_lengths(prefix_length)?;
    if fields.segment_length != segment_length {
        return Err(SegmentSealError::SegmentLength {
            expected: segment_length,
            observed: fields.segment_length,
        });
    }
    if fields.record_bytes != record_bytes {
        return Err(SegmentSealError::RecordBytes {
            expected: record_bytes,
            observed: fields.record_bytes,
        });
    }
    Ok(())
}

fn validate_algorithms(fields: &DecodedSeal) -> Result<(), SegmentSealError> {
    require_u8(
        fields.checksum_algorithm,
        ALGORITHM,
        |expected, observed| SegmentSealError::SealChecksumAlgorithm { expected, observed },
    )?;
    require_u8(fields.digest_algorithm, ALGORITHM, |expected, observed| {
        SegmentSealError::SegmentDigestAlgorithm { expected, observed }
    })?;
    let expected = [0_u8; 6];
    if fields.reserved != expected {
        return Err(SegmentSealError::ReservedBytes {
            expected,
            observed: fields.reserved,
        });
    }
    Ok(())
}

fn prefix_length(observed: usize) -> Result<u64, SegmentSealError> {
    u64::try_from(observed).map_err(|_source| SegmentSealError::PrefixLengthHostWidth { observed })
}

fn derived_lengths(bytes_before_seal: u64) -> Result<(u64, u64), SegmentSealError> {
    let segment_length = bytes_before_seal
        .checked_add(u64::from(SEAL_LENGTH))
        .ok_or(SegmentSealError::LengthArithmetic { bytes_before_seal })?;
    if segment_length > MAXIMUM_SEGMENT_LENGTH {
        return Err(SegmentSealError::SegmentLengthOutOfBounds {
            maximum: MAXIMUM_SEGMENT_LENGTH,
            observed: segment_length,
        });
    }
    let record_bytes = bytes_before_seal
        .checked_sub(u64::from(HEADER_LENGTH))
        .ok_or(SegmentSealError::LengthArithmetic { bytes_before_seal })?;
    Ok((segment_length, record_bytes))
}

fn require_u8(
    observed: u8,
    expected: u8,
    error: fn(u8, u8) -> SegmentSealError,
) -> Result<(), SegmentSealError> {
    if observed == expected {
        return Ok(());
    }
    Err(error(expected, observed))
}

fn require_u16(
    observed: u16,
    expected: u16,
    error: fn(u16, u16) -> SegmentSealError,
) -> Result<(), SegmentSealError> {
    if observed == expected {
        return Ok(());
    }
    Err(error(expected, observed))
}
