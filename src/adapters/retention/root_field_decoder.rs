//! This boundary module owns fixed-width retention root field extraction.

use std::cmp::Ordering;

use super::RetentionRootDecodeError;

pub(super) fn require_exact(
    encoded: &[u8],
    expected: usize,
) -> Result<(), RetentionRootDecodeError> {
    match encoded.len().cmp(&expected) {
        Ordering::Less => Err(RetentionRootDecodeError::Truncated {
            expected,
            observed: encoded.len(),
        }),
        Ordering::Equal => Ok(()),
        Ordering::Greater => Err(RetentionRootDecodeError::TrailingData {
            expected,
            observed: encoded.len(),
        }),
    }
}

pub(super) const fn require_minimum(
    encoded: &[u8],
    expected: usize,
) -> Result<(), RetentionRootDecodeError> {
    if encoded.len() < expected {
        Err(RetentionRootDecodeError::Truncated {
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
) -> Result<(), RetentionRootDecodeError> {
    let end = offset
        .checked_add(width)
        .ok_or(RetentionRootDecodeError::LengthOverflow)?;
    let bytes = encoded
        .get(offset..end)
        .ok_or(RetentionRootDecodeError::Truncated {
            expected: end,
            observed: encoded.len(),
        })?;
    if bytes.iter().all(|byte| *byte == 0) {
        Ok(())
    } else {
        Err(RetentionRootDecodeError::NonZeroReserved { field })
    }
}

pub(super) fn require_u16<F>(
    encoded: &[u8],
    offset: usize,
    expected: u16,
    error: F,
) -> Result<(), RetentionRootDecodeError>
where
    F: FnOnce(u16, u16) -> RetentionRootDecodeError,
{
    let observed = read_u16(encoded, offset)?;
    if observed == expected {
        Ok(())
    } else {
        Err(error(expected, observed))
    }
}

pub(super) fn read_u16(encoded: &[u8], offset: usize) -> Result<u16, RetentionRootDecodeError> {
    read_array(encoded, offset).map(u16::from_be_bytes)
}

pub(super) fn read_u32(encoded: &[u8], offset: usize) -> Result<u32, RetentionRootDecodeError> {
    read_array(encoded, offset).map(u32::from_be_bytes)
}

pub(super) fn read_u64(encoded: &[u8], offset: usize) -> Result<u64, RetentionRootDecodeError> {
    read_array(encoded, offset).map(u64::from_be_bytes)
}

pub(super) fn read_array<const WIDTH: usize>(
    encoded: &[u8],
    offset: usize,
) -> Result<[u8; WIDTH], RetentionRootDecodeError> {
    let end = offset
        .checked_add(WIDTH)
        .ok_or(RetentionRootDecodeError::LengthOverflow)?;
    let bytes = encoded
        .get(offset..end)
        .ok_or(RetentionRootDecodeError::Truncated {
            expected: end,
            observed: encoded.len(),
        })?;
    <[u8; WIDTH]>::try_from(bytes).map_err(|_| RetentionRootDecodeError::Truncated {
        expected: end,
        observed: encoded.len(),
    })
}
