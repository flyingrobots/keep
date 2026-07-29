//! Catalog strict logical-identity ordering laws.

use std::error::Error;

use keep::CatalogDecodeError;

use super::{BUNDLE_HEX, ENTRY_LENGTH, FIRST_ENTRY_OFFSET, catalog_bytes, mutation_support};

#[test]
fn catalog_refuses_duplicate_logical_identities() -> Result<(), Box<dyn Error>> {
    let mut encoded = catalog_bytes(BUNDLE_HEX)?;
    let second_offset = FIRST_ENTRY_OFFSET
        .checked_add(ENTRY_LENGTH)
        .ok_or("second-entry offset overflow")?;
    let first = entry(&encoded, FIRST_ENTRY_OFFSET)?.to_vec();
    mutation_support::replace_range(&mut encoded, second_offset, &first)?;
    mutation_support::assert_catalog_refusal(&mut encoded, |error| {
        error
            == CatalogDecodeError::DuplicateIdentity {
                first_index: 0,
                duplicate_index: 1,
            }
    })
}

#[test]
fn catalog_refuses_out_of_order_logical_identities() -> Result<(), Box<dyn Error>> {
    let mut encoded = catalog_bytes(BUNDLE_HEX)?;
    let second_offset = FIRST_ENTRY_OFFSET
        .checked_add(ENTRY_LENGTH)
        .ok_or("second-entry offset overflow")?;
    let first = entry(&encoded, FIRST_ENTRY_OFFSET)?.to_vec();
    let second = entry(&encoded, second_offset)?.to_vec();
    mutation_support::replace_range(&mut encoded, FIRST_ENTRY_OFFSET, &second)?;
    mutation_support::replace_range(&mut encoded, second_offset, &first)?;
    mutation_support::assert_catalog_refusal(&mut encoded, |error| {
        error
            == CatalogDecodeError::IdentityOrder {
                previous_index: 0,
                observed_index: 1,
            }
    })
}

fn entry(encoded: &[u8], offset: usize) -> Result<&[u8], Box<dyn Error>> {
    let end = offset
        .checked_add(ENTRY_LENGTH)
        .ok_or("entry offset overflow")?;
    encoded
        .get(offset..end)
        .ok_or_else(|| "catalog fixture lacks complete entry".into())
}
