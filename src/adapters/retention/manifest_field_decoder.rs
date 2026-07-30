//! This boundary module owns fixed-width retention manifest field extraction.

use std::cmp::Ordering;

use super::RetentionManifestDecodeError;

pub(super) fn require_exact(
    encoded: &[u8],
    expected: usize,
) -> Result<(), RetentionManifestDecodeError> {
    match encoded.len().cmp(&expected) {
        Ordering::Less => Err(RetentionManifestDecodeError::Truncated {
            expected,
            observed: encoded.len(),
        }),
        Ordering::Equal => Ok(()),
        Ordering::Greater => Err(RetentionManifestDecodeError::TrailingData {
            expected,
            observed: encoded.len(),
        }),
    }
}

pub(super) const fn require_minimum(
    encoded: &[u8],
    expected: usize,
) -> Result<(), RetentionManifestDecodeError> {
    if encoded.len() < expected {
        Err(RetentionManifestDecodeError::Truncated {
            expected,
            observed: encoded.len(),
        })
    } else {
        Ok(())
    }
}

pub(super) fn require_zero(
    encoded: &[u8],
    offset: usize,
    width: usize,
    field: &'static str,
) -> Result<(), RetentionManifestDecodeError> {
    let end = offset
        .checked_add(width)
        .ok_or(RetentionManifestDecodeError::LengthOverflow)?;
    let bytes = encoded
        .get(offset..end)
        .ok_or(RetentionManifestDecodeError::Truncated {
            expected: end,
            observed: encoded.len(),
        })?;
    if bytes.iter().all(|byte| *byte == 0) {
        Ok(())
    } else {
        Err(RetentionManifestDecodeError::NonZeroReserved { field })
    }
}

pub(super) fn require_u16<F>(
    encoded: &[u8],
    offset: usize,
    expected: u16,
    error: F,
) -> Result<(), RetentionManifestDecodeError>
where
    F: FnOnce(u16, u16) -> RetentionManifestDecodeError,
{
    let observed = read_u16(encoded, offset)?;
    if observed == expected {
        Ok(())
    } else {
        Err(error(expected, observed))
    }
}

pub(super) fn read_u16(encoded: &[u8], offset: usize) -> Result<u16, RetentionManifestDecodeError> {
    read_array(encoded, offset).map(u16::from_be_bytes)
}

pub(super) fn read_u32(encoded: &[u8], offset: usize) -> Result<u32, RetentionManifestDecodeError> {
    read_array(encoded, offset).map(u32::from_be_bytes)
}

pub(super) fn read_u64(encoded: &[u8], offset: usize) -> Result<u64, RetentionManifestDecodeError> {
    read_array(encoded, offset).map(u64::from_be_bytes)
}

pub(super) fn read_array<const WIDTH: usize>(
    encoded: &[u8],
    offset: usize,
) -> Result<[u8; WIDTH], RetentionManifestDecodeError> {
    let end = offset
        .checked_add(WIDTH)
        .ok_or(RetentionManifestDecodeError::LengthOverflow)?;
    let bytes = encoded
        .get(offset..end)
        .ok_or(RetentionManifestDecodeError::Truncated {
            expected: end,
            observed: encoded.len(),
        })?;
    <[u8; WIDTH]>::try_from(bytes).map_err(|_| RetentionManifestDecodeError::Truncated {
        expected: end,
        observed: encoded.len(),
    })
}
