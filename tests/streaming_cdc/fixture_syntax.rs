//! Canonical syntax shared by CDC conformance fixtures.

use super::harness_failure::HarnessFailure;

pub(super) fn canonical_decimal(value: &str) -> Result<usize, HarnessFailure> {
    if value != "0"
        && (value.is_empty()
            || value.starts_with('0')
            || !value.as_bytes().iter().all(u8::is_ascii_digit))
    {
        return Err(HarnessFailure::corpus("fixture decimal is noncanonical"));
    }
    value
        .parse::<usize>()
        .map_err(|_source| HarnessFailure::corpus("fixture decimal is invalid"))
}

pub(super) fn decode_hex(encoded: &str) -> Result<Vec<u8>, HarnessFailure> {
    if !encoded.len().is_multiple_of(2) {
        return Err(HarnessFailure::corpus("fixture hex has odd length"));
    }
    let capacity = encoded
        .len()
        .checked_div(2)
        .ok_or_else(|| HarnessFailure::corpus("fixture hex division failed"))?;
    let mut bytes = Vec::with_capacity(capacity);
    for pair in encoded.as_bytes().chunks_exact(2) {
        let high = pair
            .first()
            .copied()
            .ok_or_else(|| HarnessFailure::corpus("fixture hex pair is empty"))?;
        let low = pair
            .get(1)
            .copied()
            .ok_or_else(|| HarnessFailure::corpus("fixture hex pair is truncated"))?;
        bytes.push((nibble(high)? << 4) | nibble(low)?);
    }
    Ok(bytes)
}

pub(super) fn field(
    fields: &mut std::str::Split<'static, char>,
) -> Result<&'static str, HarnessFailure> {
    fields
        .next()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| HarnessFailure::corpus("fixture row is missing a field"))
}

pub(super) fn header(observed: Option<&str>, expected: &str) -> Result<(), HarnessFailure> {
    if observed == Some(expected) {
        Ok(())
    } else {
        Err(HarnessFailure::corpus("fixture header mismatch"))
    }
}

const fn nibble(value: u8) -> Result<u8, HarnessFailure> {
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
        _ => Err(HarnessFailure::corpus(
            "fixture hex is not lowercase hexadecimal",
        )),
    }
}
