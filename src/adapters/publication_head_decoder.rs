//! Canonical publication-head framing and checksum decoder.

use super::{ChecksummedPublicationHead, PublicationHeadDecodeError, framed_blake3};
use crate::{CatalogDigest, CatalogGeneration, CatalogLength};

const ENCODED_LENGTH: usize = 128;
const MAGIC: [u8; 16] = *b"KEEP:CATHEAD:V1\0";
const VERSION: u16 = 1;
const FLAGS: u16 = 0;
const HEAD_LENGTH: u16 = 128;
const ALGORITHM: u8 = 1;
const CHECKSUM_INPUT_LENGTH: usize = 96;
const CHECKSUM_DOMAIN: &[u8] = b"KEEP:CATHEAD:SUM\0";

pub(super) fn decode(
    encoded: &[u8],
) -> Result<ChecksummedPublicationHead<'_>, PublicationHeadDecodeError> {
    if encoded.len() != ENCODED_LENGTH {
        return Err(PublicationHeadDecodeError::WrongLength {
            expected: ENCODED_LENGTH,
            observed: encoded.len(),
        });
    }
    let fields = decode_fields(encoded)?;
    let (generation, catalog_length) = validate_fields(&fields)?;
    validate_checksum(encoded, fields.checksum)?;
    Ok(ChecksummedPublicationHead::from_verified_parts(
        encoded,
        generation,
        catalog_length,
        CatalogDigest::from_validated(fields.catalog_digest),
    ))
}

fn validate_fields(
    fields: &DecodedFields,
) -> Result<(CatalogGeneration, CatalogLength), PublicationHeadDecodeError> {
    require_eq(fields.magic, MAGIC, |observed| {
        PublicationHeadDecodeError::InvalidMagic { observed }
    })?;
    require_eq(fields.version, VERSION, |observed| {
        PublicationHeadDecodeError::UnsupportedVersion {
            expected: VERSION,
            observed,
        }
    })?;
    require_eq(fields.flags, FLAGS, |observed| {
        PublicationHeadDecodeError::Flags {
            expected: FLAGS,
            observed,
        }
    })?;
    require_eq(fields.head_length, HEAD_LENGTH, |observed| {
        PublicationHeadDecodeError::HeadLength {
            expected: HEAD_LENGTH,
            observed,
        }
    })?;
    require_eq(fields.checksum_algorithm, ALGORITHM, |observed| {
        PublicationHeadDecodeError::ChecksumAlgorithm {
            expected: ALGORITHM,
            observed,
        }
    })?;
    require_eq(fields.digest_algorithm, ALGORITHM, |observed| {
        PublicationHeadDecodeError::DigestAlgorithm {
            expected: ALGORITHM,
            observed,
        }
    })?;
    let generation = CatalogGeneration::new(fields.generation)
        .map_err(|source| PublicationHeadDecodeError::Generation { source })?;
    let catalog_length = CatalogLength::new(fields.catalog_length)
        .map_err(|source| PublicationHeadDecodeError::CatalogLength { source })?;
    let expected = [0_u8; 24];
    require_eq(fields.reserved, expected, |observed| {
        PublicationHeadDecodeError::Reserved { expected, observed }
    })?;
    Ok((generation, catalog_length))
}

fn validate_checksum(encoded: &[u8], observed: [u8; 32]) -> Result<(), PublicationHeadDecodeError> {
    let covered =
        encoded
            .get(..CHECKSUM_INPUT_LENGTH)
            .ok_or(PublicationHeadDecodeError::WrongLength {
                expected: ENCODED_LENGTH,
                observed: encoded.len(),
            })?;
    let expected = framed_blake3::hash(
        CHECKSUM_DOMAIN,
        &[covered],
        u64::try_from(CHECKSUM_INPUT_LENGTH).map_err(|_source| {
            PublicationHeadDecodeError::WrongLength {
                expected: ENCODED_LENGTH,
                observed: encoded.len(),
            }
        })?,
    );
    require_eq(observed, expected, |observed| {
        PublicationHeadDecodeError::ChecksumMismatch { expected, observed }
    })
}

fn decode_fields(encoded: &[u8]) -> Result<DecodedFields, PublicationHeadDecodeError> {
    Ok(DecodedFields {
        magic: read_array(encoded, 0)?,
        version: u16::from_be_bytes(read_array(encoded, 16)?),
        flags: u16::from_be_bytes(read_array(encoded, 18)?),
        head_length: u16::from_be_bytes(read_array(encoded, 20)?),
        checksum_algorithm: read_u8(encoded, 22)?,
        digest_algorithm: read_u8(encoded, 23)?,
        generation: u64::from_be_bytes(read_array(encoded, 24)?),
        catalog_length: u64::from_be_bytes(read_array(encoded, 32)?),
        catalog_digest: read_array(encoded, 40)?,
        reserved: read_array(encoded, 72)?,
        checksum: read_array(encoded, 96)?,
    })
}

fn read_u8(encoded: &[u8], offset: usize) -> Result<u8, PublicationHeadDecodeError> {
    encoded
        .get(offset)
        .copied()
        .ok_or(PublicationHeadDecodeError::WrongLength {
            expected: ENCODED_LENGTH,
            observed: encoded.len(),
        })
}

fn read_array<const LENGTH: usize>(
    encoded: &[u8],
    offset: usize,
) -> Result<[u8; LENGTH], PublicationHeadDecodeError> {
    let end = offset
        .checked_add(LENGTH)
        .ok_or(PublicationHeadDecodeError::WrongLength {
            expected: ENCODED_LENGTH,
            observed: encoded.len(),
        })?;
    encoded
        .get(offset..end)
        .and_then(|field| field.try_into().ok())
        .ok_or(PublicationHeadDecodeError::WrongLength {
            expected: ENCODED_LENGTH,
            observed: encoded.len(),
        })
}

fn require_eq<T: Copy + Eq>(
    observed: T,
    expected: T,
    error: impl FnOnce(T) -> PublicationHeadDecodeError,
) -> Result<(), PublicationHeadDecodeError> {
    if observed == expected {
        Ok(())
    } else {
        Err(error(observed))
    }
}

struct DecodedFields {
    magic: [u8; 16],
    version: u16,
    flags: u16,
    head_length: u16,
    checksum_algorithm: u8,
    digest_algorithm: u8,
    generation: u64,
    catalog_length: u64,
    catalog_digest: [u8; 32],
    reserved: [u8; 24],
    checksum: [u8; 32],
}
