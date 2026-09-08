//! This module owns migration-receipt mutation and admission test mechanics.
#![allow(
    clippy::redundant_pub_crate,
    reason = "sibling private test modules consume this harness"
)]

use std::io;

use keep::{
    AdmittedStoreFormatMarker, AdmittedStoreMigrationIntent, AdmittedStoreMigrationReceipt,
    StoreMigrationReceiptDecodeError,
};

use super::fixture::{intent_bytes, marker_bytes, receipt_bytes};

pub(super) fn assert_fixed_refusal(
    offset: usize,
    expected: StoreMigrationReceiptDecodeError,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut bytes = receipt_bytes()?;
    flip_byte(&mut bytes, offset)?;
    assert_receipt_refusal(&bytes, expected)
}

pub(super) fn assert_semantic_refusal(
    offset: usize,
    expected: StoreMigrationReceiptDecodeError,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut bytes = receipt_bytes()?;
    flip_byte(&mut bytes, offset)?;
    refresh_checksum(
        &mut bytes,
        224,
        b"keep.store-migration-receipt-checksum/v2\0",
    )?;
    assert_receipt_refusal(&bytes, expected)
}

pub(super) fn assert_receipt_refusal(
    bytes: &[u8],
    expected: StoreMigrationReceiptDecodeError,
) -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(decode_receipt(bytes)?, Err(expected));
    Ok(())
}

pub(super) fn decode_receipt(
    bytes: &[u8],
) -> Result<
    Result<AdmittedStoreMigrationReceipt<'_>, StoreMigrationReceiptDecodeError>,
    Box<dyn std::error::Error>,
> {
    let intent_bytes = intent_bytes()?;
    let marker_bytes = marker_bytes()?;
    let intent = AdmittedStoreMigrationIntent::decode(&intent_bytes)?;
    let marker = AdmittedStoreFormatMarker::decode(&marker_bytes)?;
    Ok(AdmittedStoreMigrationReceipt::decode(
        bytes, &intent, &marker,
    ))
}

pub(super) fn mutated_array<const WIDTH: usize>(
    bytes: &[u8],
    offset: usize,
    relative: usize,
) -> Result<[u8; WIDTH], io::Error> {
    let end = offset
        .checked_add(WIDTH)
        .ok_or_else(|| io::Error::other("receipt field offset overflow"))?;
    let field = bytes
        .get(offset..end)
        .ok_or_else(|| io::Error::other("receipt lacks fixed field"))?;
    let mut observed = <[u8; WIDTH]>::try_from(field)
        .map_err(|_| io::Error::other("receipt field width mismatch"))?;
    flip_byte(&mut observed, relative)?;
    Ok(observed)
}

pub(super) fn flip_byte(bytes: &mut [u8], offset: usize) -> Result<(), io::Error> {
    let byte = bytes
        .get_mut(offset)
        .ok_or_else(|| io::Error::other("receipt mutation offset is out of bounds"))?;
    *byte ^= 1;
    Ok(())
}

pub(super) fn refresh_checksum(
    bytes: &mut [u8],
    offset: usize,
    domain: &[u8],
) -> Result<(), io::Error> {
    let (preimage, checksum) = bytes
        .split_at_mut_checked(offset)
        .ok_or_else(|| io::Error::other("record lacks checksum boundary"))?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain);
    hasher.update(preimage);
    checksum.copy_from_slice(hasher.finalize().as_bytes());
    Ok(())
}

pub(super) fn digest_intent(bytes: &[u8]) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"keep.store-migration-intent/v2\0");
    hasher.update(bytes);
    *hasher.finalize().as_bytes()
}
