//! Canonical catalog checksum and physical digest verification.

use super::{CatalogDecodeError, catalog_header_decoder, framed_blake3};

const CHECKSUM_DOMAIN: &[u8] = b"KEEP:CATALOG:SUM\0";
const DIGEST_DOMAIN: &[u8] = b"KEEP:CATALOG:DIGEST\0";

pub(super) fn validate(encoded: &[u8]) -> Result<[u8; 32], CatalogDecodeError> {
    let checksum_offset = encoded
        .len()
        .checked_sub(catalog_header_decoder::TRAILER_LENGTH)
        .ok_or_else(|| minimum_length(encoded))?;
    let digest_offset = encoded
        .len()
        .checked_sub(32)
        .ok_or_else(|| minimum_length(encoded))?;
    let covered = encoded
        .get(..checksum_offset)
        .ok_or_else(|| minimum_length(encoded))?;
    let observed_checksum = read_array(encoded, checksum_offset)?;
    let expected_checksum =
        framed_blake3::hash(CHECKSUM_DOMAIN, &[covered], admitted_length(covered)?);
    if observed_checksum != expected_checksum {
        return Err(CatalogDecodeError::ChecksumMismatch {
            expected: expected_checksum,
            observed: observed_checksum,
        });
    }
    let observed_digest = read_array(encoded, digest_offset)?;
    let digest_input = encoded
        .get(..digest_offset)
        .ok_or_else(|| minimum_length(encoded))?;
    let expected_digest = framed_blake3::hash(
        DIGEST_DOMAIN,
        &[digest_input],
        admitted_length(digest_input)?,
    );
    if observed_digest != expected_digest {
        return Err(CatalogDecodeError::DigestMismatch {
            expected: expected_digest,
            observed: observed_digest,
        });
    }
    Ok(observed_digest)
}

fn admitted_length(bytes: &[u8]) -> Result<u64, CatalogDecodeError> {
    u64::try_from(bytes.len()).map_err(|_source| CatalogDecodeError::HashLength {
        observed: bytes.len(),
    })
}

fn read_array<const LENGTH: usize>(
    encoded: &[u8],
    offset: usize,
) -> Result<[u8; LENGTH], CatalogDecodeError> {
    let end = offset
        .checked_add(LENGTH)
        .ok_or_else(|| minimum_length(encoded))?;
    encoded
        .get(offset..end)
        .and_then(|field| field.try_into().ok())
        .ok_or_else(|| minimum_length(encoded))
}

const fn minimum_length(encoded: &[u8]) -> CatalogDecodeError {
    CatalogDecodeError::MinimumLength {
        minimum: catalog_header_decoder::MINIMUM_LENGTH,
        observed: encoded.len(),
    }
}
