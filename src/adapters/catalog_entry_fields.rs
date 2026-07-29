//! Bounded fixed-field extraction for one complete catalog entry.

use super::CatalogEntryDecodeError;

pub(super) const ENCODED_LENGTH: usize = 160;

pub(super) fn read_u8(encoded: &[u8], offset: usize) -> Result<u8, CatalogEntryDecodeError> {
    encoded
        .get(offset)
        .copied()
        .ok_or_else(|| wrong_length(encoded))
}

pub(super) fn read_u16(encoded: &[u8], offset: usize) -> Result<u16, CatalogEntryDecodeError> {
    Ok(u16::from_be_bytes(read_array(encoded, offset)?))
}

pub(super) fn read_u64(encoded: &[u8], offset: usize) -> Result<u64, CatalogEntryDecodeError> {
    Ok(u64::from_be_bytes(read_array(encoded, offset)?))
}

pub(super) fn read_array<const LENGTH: usize>(
    encoded: &[u8],
    offset: usize,
) -> Result<[u8; LENGTH], CatalogEntryDecodeError> {
    let end = offset
        .checked_add(LENGTH)
        .ok_or_else(|| wrong_length(encoded))?;
    encoded
        .get(offset..end)
        .and_then(|field| field.try_into().ok())
        .ok_or_else(|| wrong_length(encoded))
}

const fn wrong_length(encoded: &[u8]) -> CatalogEntryDecodeError {
    CatalogEntryDecodeError::WrongLength {
        expected: ENCODED_LENGTH,
        observed: encoded.len(),
    }
}
