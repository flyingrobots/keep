//! Canonical text `LayoutId` codec.

use std::fmt;
use std::str::FromStr;

use super::layout_id_text_error::LayoutIdTextParseError;
use super::lower_hex::{LowerHexError, decode_digest_32};
use crate::{LayoutId, LayoutRecordLength};

const SCHEME: &str = "keep";
const KIND: &str = "layout";
const VERSION: &str = "v1";
const CODEC: &str = "flat-chunks-v1";
const ALGORITHM: &str = "blake3-256";
const MAX_INPUT_BYTES: usize = 128;

impl fmt::Display for LayoutId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{SCHEME}:{KIND}:{VERSION}:{CODEC}:{ALGORITHM}:{}:",
            self.plan_length()
        )?;
        for byte in self.digest() {
            write!(formatter, "{byte:02x}")?;
        }
        Ok(())
    }
}

impl FromStr for LayoutId {
    type Err = LayoutIdTextParseError;

    fn from_str(encoded: &str) -> Result<Self, Self::Err> {
        parse_text(encoded)
    }
}

fn parse_text(encoded: &str) -> Result<LayoutId, LayoutIdTextParseError> {
    if encoded.len() > MAX_INPUT_BYTES {
        return Err(LayoutIdTextParseError::InputTooLong {
            maximum: MAX_INPUT_BYTES,
            observed: encoded.len(),
        });
    }
    let mut fields = encoded.split(':');
    let scheme = required_field(fields.next())?;
    let kind = required_field(fields.next())?;
    let version = required_field(fields.next())?;
    let codec = required_field(fields.next())?;
    let algorithm = required_field(fields.next())?;
    let length = required_field(fields.next())?;
    let digest = required_field(fields.next())?;
    if fields.next().is_some() {
        return Err(LayoutIdTextParseError::TrailingData);
    }
    validate_fixed_fields(scheme, kind, version, codec, algorithm)?;
    let plan_length = parse_plan_length(length)?;
    let digest = parse_digest(digest)?;
    Ok(LayoutId::from_validated_parts(plan_length, digest))
}

fn required_field(field: Option<&str>) -> Result<&str, LayoutIdTextParseError> {
    match field {
        Some("") | None => Err(LayoutIdTextParseError::MalformedStructure),
        Some(value) => Ok(value),
    }
}

fn validate_fixed_fields(
    scheme: &str,
    kind: &str,
    version: &str,
    codec: &str,
    algorithm: &str,
) -> Result<(), LayoutIdTextParseError> {
    if scheme != SCHEME {
        return Err(LayoutIdTextParseError::InvalidScheme);
    }
    if kind != KIND {
        return Err(LayoutIdTextParseError::InvalidKind);
    }
    validate_version(version)?;
    if codec != CODEC {
        return Err(LayoutIdTextParseError::UnsupportedCodec);
    }
    if algorithm != ALGORITHM {
        return Err(LayoutIdTextParseError::UnsupportedAlgorithm);
    }
    Ok(())
}

fn validate_version(field: &str) -> Result<(), LayoutIdTextParseError> {
    if field == VERSION {
        return Ok(());
    }
    let Some(decimal) = field.strip_prefix('v') else {
        return Err(LayoutIdTextParseError::MalformedVersion);
    };
    if !is_canonical_decimal(decimal) {
        return Err(LayoutIdTextParseError::MalformedVersion);
    }
    let observed = decimal
        .parse::<u16>()
        .map_err(|_source| LayoutIdTextParseError::MalformedVersion)?;
    Err(LayoutIdTextParseError::UnsupportedVersion { observed })
}

fn parse_plan_length(field: &str) -> Result<LayoutRecordLength, LayoutIdTextParseError> {
    if !is_canonical_decimal(field) {
        return Err(LayoutIdTextParseError::NonCanonicalPlanLength);
    }
    let observed = field
        .parse::<u64>()
        .map_err(|_source| LayoutIdTextParseError::PlanLengthOverflow)?;
    if !(LayoutRecordLength::MINIMUM..=LayoutRecordLength::MAXIMUM).contains(&observed) {
        return Err(LayoutIdTextParseError::PlanLengthOutOfBounds {
            minimum: LayoutRecordLength::MINIMUM,
            maximum: LayoutRecordLength::MAXIMUM,
            observed,
        });
    }
    LayoutRecordLength::from_wire(observed)
        .ok_or(LayoutIdTextParseError::PlanLengthNotCongruent { observed })
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

fn parse_digest(field: &str) -> Result<[u8; 32], LayoutIdTextParseError> {
    decode_digest_32(field).map_err(|error| match error {
        LowerHexError::WrongLength { expected, observed } => {
            LayoutIdTextParseError::InvalidDigestLength { expected, observed }
        }
        LowerHexError::Uppercase => LayoutIdTextParseError::NonCanonicalDigestCase,
        LowerHexError::InvalidAlphabet => LayoutIdTextParseError::InvalidDigestAlphabet,
    })
}
