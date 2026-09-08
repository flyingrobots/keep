//! This boundary module owns canonical retention-head decoding order.

use super::{ChecksummedRetentionHead, RetentionHeadDecodeError};
use crate::{LivenessGeneration, RetentionHead, RetentionManifestDigest, RetentionManifestLength};

pub(super) const ENCODED_LENGTH: usize = 144;
pub(super) const CHECKSUM_OFFSET: usize = 112;
pub(super) const MAGIC: [u8; 16] = *b"KEEP:RET:HEAD2\0\0";
pub(super) const VERSION: u16 = 2;
pub(super) const RECORD_LENGTH: u16 = 144;
const CHECKSUM_DOMAIN: &[u8] = b"keep.retention-head-checksum/v2\0";

pub(super) fn decode(
    encoded: &[u8],
) -> Result<ChecksummedRetentionHead<'_>, RetentionHeadDecodeError> {
    require_length(encoded)?;
    validate_fixed_fields(encoded)?;
    verify_checksum(encoded)?;
    let generation = LivenessGeneration::new(read_u64(encoded, 24)?)
        .map_err(|source| RetentionHeadDecodeError::LivenessGeneration { source })?;
    let manifest_length = RetentionManifestLength::new(read_u64(encoded, 32)?)
        .map_err(|source| RetentionHeadDecodeError::ManifestLength { source })?;
    let manifest_digest = RetentionManifestDigest::from_hash(read_array(encoded, 40)?);
    let predecessor = predecessor(read_array(encoded, 72)?);
    let head = RetentionHead::new(generation, manifest_length, manifest_digest, predecessor)
        .map_err(|source| RetentionHeadDecodeError::Semantic { source })?;
    Ok(ChecksummedRetentionHead::admitted(encoded, head))
}

fn validate_fixed_fields(encoded: &[u8]) -> Result<(), RetentionHeadDecodeError> {
    let magic = read_array(encoded, 0)?;
    if magic != MAGIC {
        return Err(RetentionHeadDecodeError::InvalidMagic { observed: magic });
    }
    let version = read_u16(encoded, 16)?;
    if version != VERSION {
        return Err(RetentionHeadDecodeError::UnsupportedVersion {
            expected: VERSION,
            observed: version,
        });
    }
    let record_length = read_u16(encoded, 18)?;
    if record_length != RECORD_LENGTH {
        return Err(RetentionHeadDecodeError::InvalidRecordLength {
            expected: RECORD_LENGTH,
            observed: record_length,
        });
    }
    let flags = read_u32(encoded, 20)?;
    if flags != 0 {
        return Err(RetentionHeadDecodeError::UnsupportedFlags { observed: flags });
    }
    let reserved = read_array(encoded, 104)?;
    if reserved != [0_u8; 8] {
        return Err(RetentionHeadDecodeError::NonZeroReserved { observed: reserved });
    }
    Ok(())
}

fn verify_checksum(encoded: &[u8]) -> Result<(), RetentionHeadDecodeError> {
    let preimage = encoded
        .get(..CHECKSUM_OFFSET)
        .ok_or(RetentionHeadDecodeError::WrongLength {
            expected: ENCODED_LENGTH,
            observed: encoded.len(),
        })?;
    let observed = read_array(encoded, CHECKSUM_OFFSET)?;
    let expected = checksum(preimage);
    if observed == expected {
        Ok(())
    } else {
        Err(RetentionHeadDecodeError::ChecksumMismatch { expected, observed })
    }
}

pub(super) fn checksum(preimage: &[u8]) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(CHECKSUM_DOMAIN);
    hasher.update(preimage);
    *hasher.finalize().as_bytes()
}

fn predecessor(bytes: [u8; 32]) -> Option<RetentionManifestDigest> {
    if bytes == [0_u8; 32] {
        None
    } else {
        Some(RetentionManifestDigest::from_hash(bytes))
    }
}

const fn require_length(encoded: &[u8]) -> Result<(), RetentionHeadDecodeError> {
    if encoded.len() == ENCODED_LENGTH {
        Ok(())
    } else {
        Err(RetentionHeadDecodeError::WrongLength {
            expected: ENCODED_LENGTH,
            observed: encoded.len(),
        })
    }
}

fn read_u16(encoded: &[u8], offset: usize) -> Result<u16, RetentionHeadDecodeError> {
    read_array(encoded, offset).map(u16::from_be_bytes)
}

fn read_u32(encoded: &[u8], offset: usize) -> Result<u32, RetentionHeadDecodeError> {
    read_array(encoded, offset).map(u32::from_be_bytes)
}

fn read_u64(encoded: &[u8], offset: usize) -> Result<u64, RetentionHeadDecodeError> {
    read_array(encoded, offset).map(u64::from_be_bytes)
}

fn read_array<const WIDTH: usize>(
    encoded: &[u8],
    offset: usize,
) -> Result<[u8; WIDTH], RetentionHeadDecodeError> {
    let Some(end) = offset.checked_add(WIDTH) else {
        return Err(RetentionHeadDecodeError::WrongLength {
            expected: ENCODED_LENGTH,
            observed: encoded.len(),
        });
    };
    let bytes = encoded
        .get(offset..end)
        .ok_or(RetentionHeadDecodeError::WrongLength {
            expected: ENCODED_LENGTH,
            observed: encoded.len(),
        })?;
    <[u8; WIDTH]>::try_from(bytes).map_err(|_| RetentionHeadDecodeError::WrongLength {
        expected: ENCODED_LENGTH,
        observed: encoded.len(),
    })
}
