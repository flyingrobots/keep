//! Canonical segment-seal construction from buffered or streaming prefixes.

use super::segment_digest_builder::SegmentDigestBuilder;
use super::segment_header::{HEADER_LENGTH, MAXIMUM_RECORD_COUNT, MAXIMUM_SEGMENT_LENGTH};
use super::segment_seal::{ENCODED_LENGTH, SEAL_LENGTH, SegmentSeal, SegmentSealCoordinates};
use super::segment_seal_hash::DIGEST_PREFIX_LENGTH;
use super::{SegmentDigest, SegmentSealError, segment_seal_hash};

pub(super) fn from_prefix(
    prefix: &[u8],
    record_count: u32,
) -> Result<SegmentSeal, SegmentSealError> {
    let bytes_before_seal =
        u64::try_from(prefix.len()).map_err(|_source| SegmentSealError::PrefixLengthHostWidth {
            observed: prefix.len(),
        })?;
    let coordinates = coordinates(record_count, bytes_before_seal)?;
    let digest = segment_seal_hash::segment_digest(prefix, &provisional(coordinates).encode())?;
    from_digest(coordinates, digest)
}

pub(super) fn from_digest_builder(
    record_count: u32,
    bytes_before_seal: u64,
    builder: &SegmentDigestBuilder,
) -> Result<SegmentSeal, SegmentSealError> {
    let coordinates = coordinates(record_count, bytes_before_seal)?;
    let q = provisional(coordinates).encode();
    let input_length = bytes_before_seal
        .checked_add(u64::from(DIGEST_PREFIX_LENGTH))
        .ok_or(SegmentSealError::LengthArithmetic { bytes_before_seal })?;
    let seal_prefix =
        q.get(..usize::from(DIGEST_PREFIX_LENGTH))
            .ok_or(SegmentSealError::WrongLength {
                expected: ENCODED_LENGTH,
                observed: q.len(),
            })?;
    let digest = builder.finish(seal_prefix, input_length);
    from_digest(coordinates, digest)
}

pub(super) fn derived_lengths(bytes_before_seal: u64) -> Result<(u64, u64), SegmentSealError> {
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

fn coordinates(
    record_count: u32,
    bytes_before_seal: u64,
) -> Result<SegmentSealCoordinates, SegmentSealError> {
    if record_count > MAXIMUM_RECORD_COUNT {
        return Err(SegmentSealError::RecordCountOutOfBounds {
            maximum: MAXIMUM_RECORD_COUNT,
            observed: record_count,
        });
    }
    let (segment_length, record_bytes) = derived_lengths(bytes_before_seal)?;
    Ok(SegmentSealCoordinates::new(
        record_count,
        bytes_before_seal,
        segment_length,
        record_bytes,
    ))
}

const fn provisional(coordinates: SegmentSealCoordinates) -> SegmentSeal {
    SegmentSeal::admitted(
        coordinates,
        SegmentDigest::from_validated([0_u8; 32]),
        [0_u8; 32],
    )
}

fn from_digest(
    coordinates: SegmentSealCoordinates,
    digest: SegmentDigest,
) -> Result<SegmentSeal, SegmentSealError> {
    let with_digest = SegmentSeal::admitted(coordinates, digest, [0_u8; 32]);
    let checksum = segment_seal_hash::seal_checksum(&with_digest.encode())?;
    Ok(SegmentSeal::admitted(coordinates, digest, checksum))
}
