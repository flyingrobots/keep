//! This boundary module owns store-format marker decoding order.

use super::{
    AdmittedStoreFormatMarker, StoreFormatDefinitionDigest, StoreFormatMarkerDecodeError,
    StoreFormatMarkerDigest,
};
use crate::RetentionManifest;

pub(super) const ENCODED_LENGTH: usize = 96;
pub(super) const CHECKSUM_OFFSET: usize = 64;
pub(super) const MAGIC: [u8; 16] = *b"KEEP:STORE:V2\0\0\0";
pub(super) const VERSION: u16 = 2;
pub(super) const RECORD_LENGTH: u16 = 96;
const CHECKSUM_DOMAIN: &[u8] = b"keep.segment-store-marker-checksum/v2\0";
const DIGEST_DOMAIN: &[u8] = b"keep.store-format-marker/v2\0";

pub(super) fn decode(
    encoded: &[u8],
) -> Result<AdmittedStoreFormatMarker<'_>, StoreFormatMarkerDecodeError> {
    require_length(encoded)?;
    validate_fixed_fields(encoded)?;
    verify_checksum(encoded)?;
    let definition_hash = read_array(encoded, 24)?;
    let definition_digest = StoreFormatDefinitionDigest::from_hash(definition_hash);
    if definition_digest != StoreFormatDefinitionDigest::VERSION_TWO {
        return Err(StoreFormatMarkerDecodeError::DefinitionDigestMismatch {
            expected: *StoreFormatDefinitionDigest::VERSION_TWO.as_bytes(),
            observed: definition_hash,
        });
    }
    let maximum_namespace_count = read_u32(encoded, 56)?;
    if maximum_namespace_count != RetentionManifest::MAXIMUM_ENTRY_COUNT {
        return Err(StoreFormatMarkerDecodeError::InvalidMaximumNamespaceCount {
            expected: RetentionManifest::MAXIMUM_ENTRY_COUNT,
            observed: maximum_namespace_count,
        });
    }
    Ok(AdmittedStoreFormatMarker::admitted(
        encoded,
        definition_digest,
        digest(encoded),
    ))
}

fn validate_fixed_fields(encoded: &[u8]) -> Result<(), StoreFormatMarkerDecodeError> {
    let magic = read_array(encoded, 0)?;
    if magic != MAGIC {
        return Err(StoreFormatMarkerDecodeError::InvalidMagic { observed: magic });
    }
    let version = read_u16(encoded, 16)?;
    if version != VERSION {
        return Err(StoreFormatMarkerDecodeError::UnsupportedVersion {
            expected: VERSION,
            observed: version,
        });
    }
    let record_length = read_u16(encoded, 18)?;
    if record_length != RECORD_LENGTH {
        return Err(StoreFormatMarkerDecodeError::InvalidRecordLength {
            expected: RECORD_LENGTH,
            observed: record_length,
        });
    }
    let flags = read_u32(encoded, 20)?;
    if flags != 0 {
        return Err(StoreFormatMarkerDecodeError::UnsupportedFlags { observed: flags });
    }
    let reserved = read_u32(encoded, 60)?;
    if reserved != 0 {
        return Err(StoreFormatMarkerDecodeError::NonZeroReserved { observed: reserved });
    }
    Ok(())
}

fn verify_checksum(encoded: &[u8]) -> Result<(), StoreFormatMarkerDecodeError> {
    let preimage = encoded
        .get(..CHECKSUM_OFFSET)
        .ok_or_else(|| wrong_length(encoded))?;
    let observed = read_array(encoded, CHECKSUM_OFFSET)?;
    let expected = checksum(preimage);
    if observed == expected {
        Ok(())
    } else {
        Err(StoreFormatMarkerDecodeError::ChecksumMismatch { expected, observed })
    }
}

pub(super) fn checksum(preimage: &[u8]) -> [u8; 32] {
    hash(CHECKSUM_DOMAIN, preimage)
}

pub(super) fn digest(encoded: &[u8]) -> StoreFormatMarkerDigest {
    StoreFormatMarkerDigest::from_hash(hash(DIGEST_DOMAIN, encoded))
}

fn hash(domain: &[u8], bytes: &[u8]) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain);
    hasher.update(bytes);
    *hasher.finalize().as_bytes()
}

const fn require_length(encoded: &[u8]) -> Result<(), StoreFormatMarkerDecodeError> {
    if encoded.len() == ENCODED_LENGTH {
        Ok(())
    } else {
        Err(wrong_length(encoded))
    }
}

const fn wrong_length(encoded: &[u8]) -> StoreFormatMarkerDecodeError {
    StoreFormatMarkerDecodeError::WrongLength {
        expected: ENCODED_LENGTH,
        observed: encoded.len(),
    }
}

fn read_u16(encoded: &[u8], offset: usize) -> Result<u16, StoreFormatMarkerDecodeError> {
    read_array(encoded, offset).map(u16::from_be_bytes)
}

fn read_u32(encoded: &[u8], offset: usize) -> Result<u32, StoreFormatMarkerDecodeError> {
    read_array(encoded, offset).map(u32::from_be_bytes)
}

fn read_array<const WIDTH: usize>(
    encoded: &[u8],
    offset: usize,
) -> Result<[u8; WIDTH], StoreFormatMarkerDecodeError> {
    let Some(end) = offset.checked_add(WIDTH) else {
        return Err(wrong_length(encoded));
    };
    let bytes = encoded
        .get(offset..end)
        .ok_or_else(|| wrong_length(encoded))?;
    <[u8; WIDTH]>::try_from(bytes).map_err(|_| wrong_length(encoded))
}
