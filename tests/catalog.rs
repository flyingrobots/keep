//! Public catalog framing, ordering, checksum, and digest laws.

#[path = "catalog/entry_laws.rs"]
mod entry_laws;
#[path = "catalog/format_oracle.rs"]
mod format_oracle;
#[path = "catalog/header_laws.rs"]
mod header_laws;
#[path = "catalog/integrity_laws.rs"]
mod integrity_laws;
#[path = "catalog/mutation_support.rs"]
mod mutation_support;
#[path = "catalog/ordering_laws.rs"]
mod ordering_laws;
mod support;

use std::error::Error;

use keep::ChecksummedCatalog;
use support::decode_hex;

pub(crate) const GENERATION_ONE_HEX: &str =
    include_str!("../conformance/segment-store/v1/one-zero-catalog.hex");
pub(crate) const GENERATION_TWO_HEX: &str =
    include_str!("../conformance/segment-store/v1/one-zero-catalog-generation-two.hex");
pub(crate) const BUNDLE_HEX: &str =
    include_str!("../conformance/segment-store/v1/one-zero-bundle-catalog.hex");
const GENERATION_ONE_DIGEST_HEX: &str =
    "04b82519b0399baefd0b9c0f32a871052e4c47e3a00226ab03b21661470f7320";
const GENERATION_TWO_DIGEST_HEX: &str =
    "ea7d0055fd21f00ed94809ef4e671d72fa2e6a4a5d9ecefb23f3a320a2dad993";
const BUNDLE_DIGEST_HEX: &str = "0b7cad1b6de663d34beacbc214db7497f2e36ab6b08dfbd5febbc8d06a418811";

pub(crate) const VERSION_OFFSET: usize = 16;
pub(crate) const FLAGS_OFFSET: usize = 18;
pub(crate) const HEADER_LENGTH_OFFSET: usize = 20;
pub(crate) const ENTRY_LENGTH_OFFSET: usize = 22;
pub(crate) const GENERATION_OFFSET: usize = 24;
pub(crate) const PREVIOUS_DIGEST_OFFSET: usize = 32;
pub(crate) const ENTRY_COUNT_OFFSET: usize = 64;
pub(crate) const CATALOG_LENGTH_OFFSET: usize = 72;
pub(crate) const CHECKSUM_ALGORITHM_OFFSET: usize = 80;
pub(crate) const DIGEST_ALGORITHM_OFFSET: usize = 81;
pub(crate) const HEADER_RESERVED_OFFSET: usize = 82;
pub(crate) const FIRST_ENTRY_OFFSET: usize = 128;
pub(crate) const ENTRY_FLAGS_OFFSET: usize = FIRST_ENTRY_OFFSET + 1;
pub(crate) const ENTRY_IDENTITY_LENGTH_OFFSET: usize = FIRST_ENTRY_OFFSET + 2;
pub(crate) const ENTRY_RECORD_OFFSET: usize = FIRST_ENTRY_OFFSET + 96;
pub(crate) const ENTRY_RECORD_LENGTH_OFFSET: usize = FIRST_ENTRY_OFFSET + 104;
pub(crate) const ENTRY_PAYLOAD_LENGTH_OFFSET: usize = FIRST_ENTRY_OFFSET + 112;
pub(crate) const ENTRY_RESERVED_OFFSET: usize = FIRST_ENTRY_OFFSET + 152;
pub(crate) const ENTRY_LENGTH: usize = 160;

#[test]
fn frozen_catalogs_are_checksum_and_digest_verified_exactly() -> Result<(), Box<dyn Error>> {
    let generation_one = catalog_bytes(GENERATION_ONE_HEX)?;
    let first = ChecksummedCatalog::decode(&generation_one)?;
    assert_eq!(first.generation().get(), 1);
    assert_eq!(first.previous_catalog_digest(), None);
    assert_eq!(first.entry_count(), 1);
    assert_eq!(
        first.digest().as_bytes().as_slice(),
        decode_hex(GENERATION_ONE_DIGEST_HEX)?
    );
    assert_eq!(first.encoded(), generation_one);

    let generation_two = catalog_bytes(GENERATION_TWO_HEX)?;
    let second = ChecksummedCatalog::decode(&generation_two)?;
    assert_eq!(second.generation().get(), 2);
    assert_eq!(second.previous_catalog_digest(), Some(first.digest()));
    assert_eq!(
        second.digest().as_bytes().as_slice(),
        decode_hex(GENERATION_TWO_DIGEST_HEX)?
    );

    let bundle = catalog_bytes(BUNDLE_HEX)?;
    let bundle_catalog = ChecksummedCatalog::decode(&bundle)?;
    assert_eq!(bundle_catalog.generation().get(), 1);
    assert_eq!(bundle_catalog.entry_count(), 2);
    assert_eq!(
        bundle_catalog.digest().as_bytes().as_slice(),
        decode_hex(BUNDLE_DIGEST_HEX)?
    );
    Ok(())
}

pub(crate) fn catalog_bytes(hex: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    decode_hex(
        hex.strip_suffix('\n')
            .ok_or("catalog fixture must end in one LF")?,
    )
    .map_err(Into::into)
}
