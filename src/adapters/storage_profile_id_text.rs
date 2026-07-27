//! Canonical text `StorageProfileId` codec.

use std::fmt;
use std::str::FromStr;

use super::storage_profile_id_text_error::StorageProfileIdParseError;
use crate::profile::StorageProfileId;

const SCHEME: &str = "keep";
const KIND: &str = "storage-profile";
const VERSION: &str = "v1";
const ALGORITHM: &str = "blake3-256";
const DIGEST_HEX_BYTES: usize = 64;
const MAX_TEXT_BYTES: usize =
    SCHEME.len() + KIND.len() + VERSION.len() + ALGORITHM.len() + DIGEST_HEX_BYTES + 4;

impl fmt::Display for StorageProfileId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{SCHEME}:{KIND}:{VERSION}:{ALGORITHM}:")?;
        for byte in self.digest() {
            write!(formatter, "{byte:02x}")?;
        }
        Ok(())
    }
}

impl FromStr for StorageProfileId {
    type Err = StorageProfileIdParseError;

    fn from_str(encoded: &str) -> Result<Self, Self::Err> {
        parse_text(encoded)
    }
}

fn parse_text(encoded: &str) -> Result<StorageProfileId, StorageProfileIdParseError> {
    if encoded.len() > MAX_TEXT_BYTES {
        return Err(StorageProfileIdParseError::InputTooLong {
            maximum: MAX_TEXT_BYTES,
            observed: encoded.len(),
        });
    }
    let mut fields = encoded.split(':');
    let scheme = required_field(fields.next())?;
    let kind = required_field(fields.next())?;
    let version = required_field(fields.next())?;
    let algorithm = required_field(fields.next())?;
    let digest = required_field(fields.next())?;
    if fields.next().is_some() {
        return Err(StorageProfileIdParseError::TrailingData);
    }
    validate_fixed_fields(scheme, kind, version, algorithm)?;
    let digest = parse_digest(digest)?;
    Ok(StorageProfileId::from_validated_digest(digest))
}

fn required_field(field: Option<&str>) -> Result<&str, StorageProfileIdParseError> {
    match field {
        Some("") | None => Err(StorageProfileIdParseError::MalformedStructure),
        Some(value) => Ok(value),
    }
}

fn validate_fixed_fields(
    scheme: &str,
    kind: &str,
    version: &str,
    algorithm: &str,
) -> Result<(), StorageProfileIdParseError> {
    if scheme != SCHEME {
        return Err(StorageProfileIdParseError::InvalidScheme);
    }
    if kind != KIND {
        return Err(StorageProfileIdParseError::InvalidKind);
    }
    validate_version(version)?;
    if algorithm != ALGORITHM {
        return Err(StorageProfileIdParseError::UnsupportedAlgorithm);
    }
    Ok(())
}

fn validate_version(field: &str) -> Result<(), StorageProfileIdParseError> {
    if field == VERSION {
        return Ok(());
    }
    let Some(decimal) = field.strip_prefix('v') else {
        return Err(StorageProfileIdParseError::MalformedVersion);
    };
    if decimal.is_empty() || !decimal.as_bytes().iter().all(u8::is_ascii_digit) {
        return Err(StorageProfileIdParseError::MalformedVersion);
    }
    let observed = decimal
        .parse::<u16>()
        .map_err(|_source| StorageProfileIdParseError::MalformedVersion)?;
    Err(StorageProfileIdParseError::UnsupportedVersion { observed })
}

fn parse_digest(field: &str) -> Result<[u8; 32], StorageProfileIdParseError> {
    if field.len() != DIGEST_HEX_BYTES {
        return Err(StorageProfileIdParseError::InvalidDigestLength {
            expected: DIGEST_HEX_BYTES,
            observed: field.len(),
        });
    }
    let mut digest = [0_u8; 32];
    for (slot, pair) in digest.iter_mut().zip(field.as_bytes().chunks_exact(2)) {
        let high = pair
            .first()
            .copied()
            .ok_or_else(|| invalid_length(field.len()))?;
        let low = pair
            .get(1)
            .copied()
            .ok_or_else(|| invalid_length(field.len()))?;
        let high_nibble = decode_nibble(high)?;
        let shifted = high_nibble
            .checked_shl(4)
            .ok_or(StorageProfileIdParseError::InvalidDigestAlphabet)?;
        *slot = shifted | decode_nibble(low)?;
    }
    Ok(digest)
}

const fn invalid_length(observed: usize) -> StorageProfileIdParseError {
    StorageProfileIdParseError::InvalidDigestLength {
        expected: DIGEST_HEX_BYTES,
        observed,
    }
}

const fn decode_nibble(value: u8) -> Result<u8, StorageProfileIdParseError> {
    match value {
        b'0' => Ok(0),
        b'1' => Ok(1),
        b'2' => Ok(2),
        b'3' => Ok(3),
        b'4' => Ok(4),
        b'5' => Ok(5),
        b'6' => Ok(6),
        b'7' => Ok(7),
        b'8' => Ok(8),
        b'9' => Ok(9),
        b'a' => Ok(10),
        b'b' => Ok(11),
        b'c' => Ok(12),
        b'd' => Ok(13),
        b'e' => Ok(14),
        b'f' => Ok(15),
        b'A'..=b'F' => Err(StorageProfileIdParseError::NonCanonicalDigestCase),
        _ => Err(StorageProfileIdParseError::InvalidDigestAlphabet),
    }
}
