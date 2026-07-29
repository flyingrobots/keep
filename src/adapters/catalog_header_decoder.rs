//! Fixed-width version-1 catalog-header field decoder.

use super::CatalogDecodeError;

pub(super) const HEADER_LENGTH: u16 = 128;
pub(super) const HEADER_LENGTH_BYTES: usize = 128;
pub(super) const ENTRY_LENGTH: u16 = 160;
pub(super) const TRAILER_LENGTH: usize = 64;
pub(super) const MINIMUM_LENGTH: usize = HEADER_LENGTH_BYTES + TRAILER_LENGTH;

pub(super) struct DecodedCatalogHeader {
    pub(super) magic: [u8; 16],
    pub(super) version: u16,
    pub(super) flags: u16,
    pub(super) header_length: u16,
    pub(super) entry_length: u16,
    pub(super) generation: u64,
    pub(super) previous_digest: [u8; 32],
    pub(super) entry_count: u64,
    pub(super) catalog_length: u64,
    pub(super) checksum_algorithm: u8,
    pub(super) digest_algorithm: u8,
    pub(super) reserved: [u8; 46],
}

pub(super) fn decode(encoded: &[u8]) -> Result<DecodedCatalogHeader, CatalogDecodeError> {
    if encoded.len() < MINIMUM_LENGTH {
        return Err(CatalogDecodeError::MinimumLength {
            minimum: MINIMUM_LENGTH,
            observed: encoded.len(),
        });
    }
    Ok(DecodedCatalogHeader {
        magic: read_array(encoded, 0)?,
        version: u16::from_be_bytes(read_array(encoded, 16)?),
        flags: u16::from_be_bytes(read_array(encoded, 18)?),
        header_length: u16::from_be_bytes(read_array(encoded, 20)?),
        entry_length: u16::from_be_bytes(read_array(encoded, 22)?),
        generation: u64::from_be_bytes(read_array(encoded, 24)?),
        previous_digest: read_array(encoded, 32)?,
        entry_count: u64::from_be_bytes(read_array(encoded, 64)?),
        catalog_length: u64::from_be_bytes(read_array(encoded, 72)?),
        checksum_algorithm: read_u8(encoded, 80)?,
        digest_algorithm: read_u8(encoded, 81)?,
        reserved: read_array(encoded, 82)?,
    })
}

fn read_u8(encoded: &[u8], offset: usize) -> Result<u8, CatalogDecodeError> {
    encoded
        .get(offset)
        .copied()
        .ok_or(CatalogDecodeError::MinimumLength {
            minimum: MINIMUM_LENGTH,
            observed: encoded.len(),
        })
}

fn read_array<const LENGTH: usize>(
    encoded: &[u8],
    offset: usize,
) -> Result<[u8; LENGTH], CatalogDecodeError> {
    let end = offset
        .checked_add(LENGTH)
        .ok_or(CatalogDecodeError::MinimumLength {
            minimum: MINIMUM_LENGTH,
            observed: encoded.len(),
        })?;
    encoded
        .get(offset..end)
        .and_then(|field| field.try_into().ok())
        .ok_or(CatalogDecodeError::MinimumLength {
            minimum: MINIMUM_LENGTH,
            observed: encoded.len(),
        })
}
