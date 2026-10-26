//! This boundary module owns fixed-width GC retirement intent field
//! extraction.

use std::cmp::Ordering;

use super::GcRetirementIntentDecodeError as Error;

pub(super) fn require_exact(encoded: &[u8], expected: usize) -> Result<(), Error> {
    match encoded.len().cmp(&expected) {
        Ordering::Less => Err(Error::Truncated {
            expected,
            observed: encoded.len(),
        }),
        Ordering::Equal => Ok(()),
        Ordering::Greater => Err(Error::TrailingData {
            expected,
            observed: encoded.len(),
        }),
    }
}

pub(super) const fn require_minimum(encoded: &[u8], expected: usize) -> Result<(), Error> {
    if encoded.len() < expected {
        Err(Error::Truncated {
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
) -> Result<(), Error> {
    let end = offset.checked_add(width).ok_or(Error::LengthOverflow)?;
    let bytes = encoded.get(offset..end).ok_or(Error::Truncated {
        expected: end,
        observed: encoded.len(),
    })?;
    if bytes.iter().all(|byte| *byte == 0) {
        Ok(())
    } else {
        Err(Error::NonZeroReserved { field })
    }
}

pub(super) fn require_u16<F>(
    encoded: &[u8],
    offset: usize,
    expected: u16,
    error: F,
) -> Result<(), Error>
where
    F: FnOnce(u16, u16) -> Error,
{
    let observed = read_u16(encoded, offset)?;
    if observed == expected {
        Ok(())
    } else {
        Err(error(expected, observed))
    }
}

pub(super) fn read_u16(encoded: &[u8], offset: usize) -> Result<u16, Error> {
    read_array(encoded, offset).map(u16::from_be_bytes)
}

pub(super) fn read_u32(encoded: &[u8], offset: usize) -> Result<u32, Error> {
    read_array(encoded, offset).map(u32::from_be_bytes)
}

pub(super) fn read_u64(encoded: &[u8], offset: usize) -> Result<u64, Error> {
    read_array(encoded, offset).map(u64::from_be_bytes)
}

pub(super) fn read_array<const WIDTH: usize>(
    encoded: &[u8],
    offset: usize,
) -> Result<[u8; WIDTH], Error> {
    let end = offset.checked_add(WIDTH).ok_or(Error::LengthOverflow)?;
    let bytes = encoded.get(offset..end).ok_or(Error::Truncated {
        expected: end,
        observed: encoded.len(),
    })?;
    <[u8; WIDTH]>::try_from(bytes).map_err(|_| Error::Truncated {
        expected: end,
        observed: encoded.len(),
    })
}
