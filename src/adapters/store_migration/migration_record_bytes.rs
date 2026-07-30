//! This boundary module owns fixed-width store-migration record field access.

use super::{StoreMigrationIntentDecodeError, StoreMigrationReceiptDecodeError};

const ENCODED_LENGTH: usize = 256;

pub(super) trait MigrationRecordDecodeError: Sized {
    fn wrong_length(expected: usize, observed: usize) -> Self;
}

impl MigrationRecordDecodeError for StoreMigrationIntentDecodeError {
    fn wrong_length(expected: usize, observed: usize) -> Self {
        Self::WrongLength { expected, observed }
    }
}

impl MigrationRecordDecodeError for StoreMigrationReceiptDecodeError {
    fn wrong_length(expected: usize, observed: usize) -> Self {
        Self::WrongLength { expected, observed }
    }
}

pub(super) fn require_length<Error: MigrationRecordDecodeError>(
    encoded: &[u8],
) -> Result<(), Error> {
    if encoded.len() == ENCODED_LENGTH {
        Ok(())
    } else {
        Err(wrong_length(encoded))
    }
}

pub(super) fn wrong_length<Error: MigrationRecordDecodeError>(encoded: &[u8]) -> Error {
    Error::wrong_length(ENCODED_LENGTH, encoded.len())
}

pub(super) fn read_u16<Error: MigrationRecordDecodeError>(
    encoded: &[u8],
    offset: usize,
) -> Result<u16, Error> {
    read_array(encoded, offset).map(u16::from_be_bytes)
}

pub(super) fn read_u32<Error: MigrationRecordDecodeError>(
    encoded: &[u8],
    offset: usize,
) -> Result<u32, Error> {
    read_array(encoded, offset).map(u32::from_be_bytes)
}

pub(super) fn read_u64<Error: MigrationRecordDecodeError>(
    encoded: &[u8],
    offset: usize,
) -> Result<u64, Error> {
    read_array(encoded, offset).map(u64::from_be_bytes)
}

pub(super) fn read_array<const WIDTH: usize, Error: MigrationRecordDecodeError>(
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
