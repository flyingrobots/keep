use std::collections::BTreeSet;

use super::GoldenError;

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
    if value.is_empty() && empty == EmptyHex::Refuse {
        return Err(GoldenError::violation(format!(
            "{field}: empty hexadecimal value"
        )));
    }
    let maximum_digits = maximum_bytes
        .checked_mul(2)
        .ok_or_else(|| GoldenError::violation(format!("{field}: hexadecimal bound overflow")))?;
    if value.len() > maximum_digits || !value.len().is_multiple_of(2) {
        return Err(GoldenError::violation(format!(
            "{field}: hexadecimal length is invalid or unbounded"
        )));
    }
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| decode_hex_pair(pair, field))
        .collect()
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) enum EmptyHex {
    Allow,
    Refuse,
}

fn decode_hex_pair(pair: &[u8], field: &str) -> Result<u8, GoldenError> {
    let [high_byte, low_byte] = pair else {
        return Err(GoldenError::violation(format!(
            "{field}: hexadecimal length is invalid"
        )));
    };
    let high = hex_nibble(*high_byte).ok_or_else(|| {
        GoldenError::violation(format!(
            "{field}: hexadecimal value is not canonical lowercase"
        ))
    })?;
    let low = hex_nibble(*low_byte).ok_or_else(|| {
        GoldenError::violation(format!(
            "{field}: hexadecimal value is not canonical lowercase"
        ))
    })?;
    high.checked_mul(16)
        .and_then(|shifted| shifted.checked_add(low))
        .ok_or_else(|| GoldenError::violation(format!("{field}: hexadecimal byte overflow")))
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

fn hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => byte.checked_sub(b'0'),
        b'a'..=b'f' => byte
            .checked_sub(b'a')
            .and_then(|offset| offset.checked_add(10)),
        _ => None,
    }
}
