//! Canonical text `BlobId` codec.

use std::fmt;
use std::str::FromStr;

use super::blob_id_text_error::BlobIdTextParseError;
use super::lower_hex::{LowerHexError, decode_digest_32};
use crate::blob::{BlobId, BlobLength};

const SCHEME: &str = "keep";
const KIND: &str = "blob";
const VERSION: &str = "v1";
const ALGORITHM: &str = "blake3-256";
const MAX_TEXT_BYTES: usize = 109;

impl fmt::Display for BlobId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{SCHEME}:{KIND}:{VERSION}:{ALGORITHM}:{}:",
            self.logical_length()
        )?;
        for byte in self.digest() {
            write!(formatter, "{byte:02x}")?;
        }
        Ok(())
    }
}

impl FromStr for BlobId {
    type Err = BlobIdTextParseError;

    fn from_str(encoded: &str) -> Result<Self, Self::Err> {
        parse_text(encoded)
    }
}

fn parse_text(encoded: &str) -> Result<BlobId, BlobIdTextParseError> {
    if encoded.len() > MAX_TEXT_BYTES {
        return Err(BlobIdTextParseError::InputTooLong {
            maximum: MAX_TEXT_BYTES,
            observed: encoded.len(),
        });
    }
    let mut fields = encoded.split(':');
    let scheme = required_field(fields.next())?;
    let kind = required_field(fields.next())?;
    let version = required_field(fields.next())?;
    let algorithm = required_field(fields.next())?;
    let length = required_field(fields.next())?;
    let digest = required_field(fields.next())?;
    if fields.next().is_some() {
        return Err(BlobIdTextParseError::TrailingData);
    }
    if scheme != SCHEME {
        return Err(BlobIdTextParseError::InvalidScheme);
    }
    if kind != KIND {
        return Err(BlobIdTextParseError::InvalidKind);
    }
    validate_version(version)?;
    if algorithm != ALGORITHM {
        return Err(BlobIdTextParseError::UnsupportedAlgorithm);
    }
    let logical_length = parse_length(length)?;
    let digest = parse_digest(digest)?;
    Ok(BlobId::from_validated_parts(logical_length, digest))
}

fn required_field(field: Option<&str>) -> Result<&str, BlobIdTextParseError> {
    match field {
        Some("") | None => Err(BlobIdTextParseError::MissingField),
        Some(value) => Ok(value),
    }
}

fn validate_version(field: &str) -> Result<(), BlobIdTextParseError> {
    if field == VERSION {
        return Ok(());
    }
    let Some(decimal) = field.strip_prefix('v') else {
        return Err(BlobIdTextParseError::MalformedVersion);
    };
    if !is_canonical_decimal(decimal) {
        return Err(BlobIdTextParseError::MalformedVersion);
    }
    let observed = decimal
        .parse::<u16>()
        .map_err(|_source| BlobIdTextParseError::MalformedVersion)?;
    Err(BlobIdTextParseError::UnsupportedVersion { observed })
}

fn parse_length(field: &str) -> Result<BlobLength, BlobIdTextParseError> {
    if !is_canonical_decimal(field) {
        return Err(BlobIdTextParseError::NonCanonicalLength);
    }
    let value = field
        .parse::<u64>()
        .map_err(|_source| BlobIdTextParseError::LengthOverflow)?;
    Ok(BlobLength::new(value))
}

fn is_canonical_decimal(field: &str) -> bool {
    if field == "0" {
        return true;
    }
    let Some(first) = field.as_bytes().first() else {
        return false;
    };
    (b'1'..=b'9').contains(first) && field.as_bytes().iter().all(u8::is_ascii_digit)
}

fn parse_digest(field: &str) -> Result<[u8; 32], BlobIdTextParseError> {
    decode_digest_32(field).map_err(|error| match error {
        LowerHexError::WrongLength { expected, observed } => {
            BlobIdTextParseError::InvalidDigestLength { expected, observed }
        }
        LowerHexError::Uppercase => BlobIdTextParseError::NonCanonicalDigestCase,
        LowerHexError::InvalidAlphabet => BlobIdTextParseError::InvalidDigestAlphabet,
    })
}
