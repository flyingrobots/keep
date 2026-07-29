//! Streaming canonical-order validation for catalog entries.

use std::cmp::Ordering;

use super::{
    CatalogDecodeError, SegmentRecordIdentity, catalog_entry_decoder, catalog_header_decoder,
};

pub(super) fn validate(encoded: &[u8], entry_count: u64) -> Result<(), CatalogDecodeError> {
    let entries_end = encoded
        .len()
        .checked_sub(catalog_header_decoder::TRAILER_LENGTH)
        .ok_or(CatalogDecodeError::MinimumLength {
            minimum: catalog_header_decoder::MINIMUM_LENGTH,
            observed: encoded.len(),
        })?;
    let entries = encoded
        .get(catalog_header_decoder::HEADER_LENGTH_BYTES..entries_end)
        .ok_or(CatalogDecodeError::MinimumLength {
            minimum: catalog_header_decoder::MINIMUM_LENGTH,
            observed: encoded.len(),
        })?;
    let mut previous: Option<(u64, SegmentRecordIdentity)> = None;
    for (host_index, entry) in entries
        .chunks_exact(catalog_entry_decoder::ENCODED_LENGTH)
        .enumerate()
    {
        let index = u64::try_from(host_index)
            .map_err(|_source| CatalogDecodeError::LengthArithmetic { entry_count })?;
        let identity = catalog_entry_decoder::decode(entry)
            .map_err(|source| CatalogDecodeError::Entry { index, source })?;
        validate_order(previous, index, identity)?;
        previous = Some((index, identity));
    }
    Ok(())
}

fn validate_order(
    previous: Option<(u64, SegmentRecordIdentity)>,
    observed_index: u64,
    observed: SegmentRecordIdentity,
) -> Result<(), CatalogDecodeError> {
    let Some((previous_index, previous_identity)) = previous else {
        return Ok(());
    };
    match previous_identity.cmp(&observed) {
        Ordering::Less => Ok(()),
        Ordering::Equal => Err(CatalogDecodeError::DuplicateIdentity {
            first_index: previous_index,
            duplicate_index: observed_index,
        }),
        Ordering::Greater => Err(CatalogDecodeError::IdentityOrder {
            previous_index,
            observed_index,
        }),
    }
}
