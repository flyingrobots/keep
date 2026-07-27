//! Canonical text `StorageProfileId` codec.

use std::fmt;
use std::str::FromStr;

use super::lower_hex::{LowerHexError, decode_digest_32};
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
    decode_digest_32(field).map_err(|error| match error {
        LowerHexError::WrongLength { expected, observed } => {
            StorageProfileIdParseError::InvalidDigestLength { expected, observed }
        }
        LowerHexError::Uppercase => StorageProfileIdParseError::NonCanonicalDigestCase,
        LowerHexError::InvalidAlphabet => StorageProfileIdParseError::InvalidDigestAlphabet,
    })
}
