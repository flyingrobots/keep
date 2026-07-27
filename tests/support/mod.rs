//! Shared deterministic integration-corpus support.

use std::io;

/// Decodes exact lowercase hexadecimal fixture transport.
///
/// # Errors
///
/// Returns an I/O-shaped corpus error for odd width, invalid digits, or
/// impossible checked arithmetic.
pub fn decode_hex(encoded: &str) -> Result<Vec<u8>, io::Error> {
    if !encoded.len().is_multiple_of(2) {
        return Err(invalid_corpus("hex input has odd length"));
    }
    let capacity = encoded
        .len()
        .checked_div(2)
        .ok_or_else(|| invalid_corpus("hexadecimal width divisor is zero"))?;
    let mut decoded = Vec::with_capacity(capacity);
    for pair in encoded.as_bytes().chunks_exact(2) {
        let high = pair
            .first()
            .copied()
            .and_then(hex_nibble)
            .ok_or_else(|| invalid_corpus("invalid hexadecimal input"))?;
        let low = pair
            .get(1)
            .copied()
            .and_then(hex_nibble)
            .ok_or_else(|| invalid_corpus("invalid hexadecimal input"))?;
        let byte = high
            .checked_shl(4)
            .and_then(|shifted| shifted.checked_add(low))
            .ok_or_else(|| invalid_corpus("hexadecimal byte overflow"))?;
        decoded.push(byte);
    }
    Ok(decoded)
}

/// Constructs a deterministic malformed-corpus failure.
pub fn invalid_corpus(message: &'static str) -> io::Error {
    io::Error::other(message)
}

const fn hex_nibble(value: u8) -> Option<u8> {
    match value {
        b'0' => Some(0),
        b'1' => Some(1),
        b'2' => Some(2),
        b'3' => Some(3),
        b'4' => Some(4),
        b'5' => Some(5),
        b'6' => Some(6),
        b'7' => Some(7),
        b'8' => Some(8),
        b'9' => Some(9),
        b'a' => Some(10),
        b'b' => Some(11),
        b'c' => Some(12),
        b'd' => Some(13),
        b'e' => Some(14),
        b'f' => Some(15),
        _ => None,
    }
}
