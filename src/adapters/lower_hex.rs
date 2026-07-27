//! Exact lowercase hexadecimal decoding for fixed identity digests.

/// Failure to decode one fixed-width lowercase hexadecimal digest.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum LowerHexError {
    /// The encoded digest width differs from the required width.
    WrongLength {
        /// Required hexadecimal character count.
        expected: usize,
        /// Observed character count.
        observed: usize,
    },
    /// Uppercase hexadecimal appeared in an otherwise hexadecimal digest.
    Uppercase,
    /// A character is outside the lowercase hexadecimal alphabet.
    InvalidAlphabet,
}

pub(super) fn decode_digest_32(field: &str) -> Result<[u8; 32], LowerHexError> {
    const ENCODED_LENGTH: usize = 64;

    if field.len() != ENCODED_LENGTH {
        return Err(LowerHexError::WrongLength {
            expected: ENCODED_LENGTH,
            observed: field.len(),
        });
    }
    let mut digest = [0_u8; 32];
    for (slot, pair) in digest.iter_mut().zip(field.as_bytes().chunks_exact(2)) {
        let high = pair.first().copied().ok_or(LowerHexError::WrongLength {
            expected: ENCODED_LENGTH,
            observed: field.len(),
        })?;
        let low = pair.get(1).copied().ok_or(LowerHexError::WrongLength {
            expected: ENCODED_LENGTH,
            observed: field.len(),
        })?;
        let high_nibble = decode_nibble(high)?;
        let shifted = high_nibble
            .checked_shl(4)
            .ok_or(LowerHexError::InvalidAlphabet)?;
        *slot = shifted | decode_nibble(low)?;
    }
    Ok(digest)
}

const fn decode_nibble(value: u8) -> Result<u8, LowerHexError> {
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
        b'A'..=b'F' => Err(LowerHexError::Uppercase),
        _ => Err(LowerHexError::InvalidAlphabet),
    }
}
