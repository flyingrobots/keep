//! This boundary module owns fixed-width GC retirement receipt field access.

use super::GcRetirementReceiptDecodeError as Error;
use super::receipt_format::ENCODED_LENGTH;

pub(super) const fn require_length(encoded: &[u8]) -> Result<(), Error> {
    if encoded.len() == ENCODED_LENGTH {
        Ok(())
    } else {
        Err(wrong_length(encoded))
    }
}

pub(super) const fn wrong_length(encoded: &[u8]) -> Error {
    Error::WrongLength {
        expected: ENCODED_LENGTH,
        observed: encoded.len(),
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
    let Some(end) = offset.checked_add(WIDTH) else {
        return Err(wrong_length(encoded));
    };
    let bytes = encoded
        .get(offset..end)
        .ok_or_else(|| wrong_length(encoded))?;
    <[u8; WIDTH]>::try_from(bytes).map_err(|_| wrong_length(encoded))
}
