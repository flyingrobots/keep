//! This module owns canonical scalar parsing and identifier uniqueness checks.

use std::collections::BTreeSet;

use super::GoldenError;
use xtask::protocol_admission::{HexError, decode_lower_hex};

pub(super) use xtask::protocol_admission::EmptyHex;

pub(super) fn case_name<'a>(value: &'a str, table: &str) -> Result<&'a str, GoldenError> {
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
        Err(GoldenError::violation(format!(
            "{table}: noncanonical case identifier {value:?}"
        )))
    }
}

pub(super) fn unique(
    value: &str,
    seen: &mut BTreeSet<String>,
    table: &str,
) -> Result<(), GoldenError> {
    if seen.insert(value.to_owned()) {
        Ok(())
    } else {
        Err(GoldenError::violation(format!(
            "{table}: duplicate identifier {value:?}"
        )))
    }
}

pub(super) fn decimal(value: &str, field: &str, maximum: u64) -> Result<u64, GoldenError> {
    if !canonical_decimal(value) {
        return Err(GoldenError::violation(format!(
            "noncanonical {field}: {value:?}"
        )));
    }
    if decimal_exceeds(value, maximum) {
        return Err(GoldenError::violation(format!(
            "{field} exceeds {maximum}: {value}"
        )));
    }
    value.parse::<u64>().map_err(|source| GoldenError::Integer {
        field: field.to_owned(),
        source,
    })
}

pub(super) fn decoded_hex(
    value: &str,
    field: &str,
    maximum_bytes: usize,
    empty: EmptyHex,
) -> Result<Vec<u8>, GoldenError> {
    decode_lower_hex(value, maximum_bytes, empty).map_err(|error| match error {
        HexError::Empty => GoldenError::violation(format!("{field}: empty hexadecimal value")),
        HexError::BoundOverflow => {
            GoldenError::violation(format!("{field}: hexadecimal bound overflow"))
        }
        HexError::InvalidLength => GoldenError::violation(format!(
            "{field}: hexadecimal length is invalid or unbounded"
        )),
        HexError::NonCanonicalAlphabet => GoldenError::violation(format!(
            "{field}: hexadecimal value is not canonical lowercase"
        )),
        HexError::ByteOverflow => {
            GoldenError::violation(format!("{field}: hexadecimal byte overflow"))
        }
    })
}

fn canonical_decimal(value: &str) -> bool {
    value == "0"
        || (!value.is_empty()
            && value
                .as_bytes()
                .first()
                .is_some_and(|byte| matches!(byte, b'1'..=b'9'))
            && value.bytes().all(|byte| byte.is_ascii_digit()))
}

fn decimal_exceeds(value: &str, maximum: u64) -> bool {
    let maximum = maximum.to_string();
    value.len() > maximum.len() || (value.len() == maximum.len() && value > maximum.as_str())
}
