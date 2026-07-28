//! This module owns shared canonical scalar admission for conformance tables.

use xtask::protocol_admission::{EmptyHex, HexError, decode_lower_hex};

use super::ConformanceError;

pub(super) fn case_name<'a>(value: &'a str, table: &str) -> Result<&'a str, ConformanceError> {
    let bytes = value.as_bytes();
    let canonical = !bytes.is_empty()
        && bytes.first().is_some_and(u8::is_ascii_alphanumeric)
        && bytes.last().is_some_and(u8::is_ascii_alphanumeric)
        && bytes
            .iter()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'-')
        && !bytes.windows(2).any(|pair| pair == b"--");
    if canonical {
        Ok(value)
    } else {
        Err(ConformanceError::violation(format!(
            "{table}: noncanonical case name {value:?}"
        )))
    }
}

pub(super) fn decimal(value: &str, field: &str, maximum: usize) -> Result<usize, ConformanceError> {
    let canonical = value == "0"
        || (!value.is_empty()
            && value
                .as_bytes()
                .first()
                .is_some_and(|byte| matches!(byte, b'1'..=b'9'))
            && value.bytes().all(|byte| byte.is_ascii_digit()));
    if !canonical {
        return Err(ConformanceError::violation(format!(
            "{field}: noncanonical unsigned decimal {value:?}"
        )));
    }
    let parsed = value
        .parse::<usize>()
        .map_err(|source| ConformanceError::Integer {
            field: field.to_owned(),
            source,
        })?;
    if parsed > maximum {
        return Err(ConformanceError::violation(format!(
            "{field}: {parsed} exceeds {maximum}"
        )));
    }
    Ok(parsed)
}

pub(super) fn lower_hex(
    value: &str,
    field: &str,
    maximum: usize,
    empty: EmptyHex,
) -> Result<Vec<u8>, ConformanceError> {
    decode_lower_hex(value, maximum, empty).map_err(|error| hex_error(field, error))
}

pub(super) fn exact_hex(
    value: &str,
    field: &str,
    exact: usize,
) -> Result<Vec<u8>, ConformanceError> {
    let decoded = lower_hex(value, field, exact, EmptyHex::Refuse)?;
    if decoded.len() != exact {
        return Err(ConformanceError::violation(format!(
            "{field}: expected {exact} bytes, observed {}",
            decoded.len()
        )));
    }
    Ok(decoded)
}

fn hex_error(field: &str, error: HexError) -> ConformanceError {
    ConformanceError::violation(format!(
        "{field}: value is not canonical lowercase hexadecimal: {error}"
    ))
}
