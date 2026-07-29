//! Integrity-valid catalog mutation support.
#![allow(
    clippy::redundant_pub_crate,
    reason = "sibling law modules share these private test fixtures"
)]

use std::error::Error;

use keep::{CatalogDecodeError, ChecksummedCatalog};

use super::{GENERATION_ONE_HEX, catalog_bytes, format_oracle};
use crate::support::require_error;

pub(crate) fn assert_byte_refusal(
    offset: usize,
    value: u8,
    predicate: impl FnOnce(CatalogDecodeError) -> bool,
) -> Result<(), Box<dyn Error>> {
    let mut encoded = catalog_bytes(GENERATION_ONE_HEX)?;
    *encoded
        .get_mut(offset)
        .ok_or("catalog fixture lacks mutation offset")? = value;
    assert_catalog_refusal(&mut encoded, predicate)
}

pub(crate) fn assert_u16_refusal(
    offset: usize,
    value: u16,
    predicate: impl FnOnce(CatalogDecodeError) -> bool,
) -> Result<(), Box<dyn Error>> {
    let mut encoded = catalog_bytes(GENERATION_ONE_HEX)?;
    replace_range(&mut encoded, offset, &value.to_be_bytes())?;
    assert_catalog_refusal(&mut encoded, predicate)
}

pub(crate) fn assert_u64_refusal(
    offset: usize,
    value: u64,
    predicate: impl FnOnce(CatalogDecodeError) -> bool,
) -> Result<(), Box<dyn Error>> {
    let mut encoded = catalog_bytes(GENERATION_ONE_HEX)?;
    replace_range(&mut encoded, offset, &value.to_be_bytes())?;
    assert_catalog_refusal(&mut encoded, predicate)
}

pub(crate) fn assert_catalog_refusal(
    encoded: &mut [u8],
    predicate: impl FnOnce(CatalogDecodeError) -> bool,
) -> Result<(), Box<dyn Error>> {
    format_oracle::seal(encoded)?;
    let error = require_error(
        ChecksummedCatalog::decode(encoded),
        "mutated catalog was admitted",
    )?;
    assert!(predicate(error), "unexpected refusal: {error:?}");
    Ok(())
}

pub(crate) fn replace_range(
    target: &mut [u8],
    offset: usize,
    replacement: &[u8],
) -> Result<(), Box<dyn Error>> {
    let end = offset
        .checked_add(replacement.len())
        .ok_or("test mutation offset overflow")?;
    target
        .get_mut(offset..end)
        .ok_or("catalog fixture lacks mutation field")?
        .copy_from_slice(replacement);
    Ok(())
}

pub(crate) fn zero_range(
    target: &mut [u8],
    offset: usize,
    length: usize,
) -> Result<(), Box<dyn Error>> {
    let end = offset
        .checked_add(length)
        .ok_or("test zeroing offset overflow")?;
    target
        .get_mut(offset..end)
        .ok_or("catalog fixture lacks zeroing field")?
        .fill(0);
    Ok(())
}
