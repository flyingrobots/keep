//! Catalog width, checksum, and digest refusal laws.

use std::error::Error;

use keep::{CatalogDecodeError, ChecksummedCatalog};

use super::{FIRST_ENTRY_OFFSET, GENERATION_ONE_HEX, catalog_bytes};
use crate::support::require_error;

#[test]
fn catalog_integrity_precedes_entry_semantics() -> Result<(), Box<dyn Error>> {
    let mut encoded = catalog_bytes(GENERATION_ONE_HEX)?;
    *encoded
        .get_mut(FIRST_ENTRY_OFFSET)
        .ok_or("catalog lacks its first entry")? = 3;

    let error = require_error(
        ChecksummedCatalog::decode(&encoded),
        "catalog with stale integrity fields was admitted",
    )?;
    assert!(
        matches!(error, CatalogDecodeError::ChecksumMismatch { .. }),
        "unexpected refusal: {error:?}"
    );
    Ok(())
}

#[test]
fn catalog_refuses_wrong_width_checksum_and_digest() -> Result<(), Box<dyn Error>> {
    let mut truncated = catalog_bytes(GENERATION_ONE_HEX)?;
    let _last = truncated.pop().ok_or("catalog fixture is empty")?;
    assert_eq!(
        require_error(
            ChecksummedCatalog::decode(&truncated),
            "truncated catalog was admitted"
        )?,
        CatalogDecodeError::ObservedLength {
            declared: 352,
            observed: 351,
        }
    );

    let mut checksum = catalog_bytes(GENERATION_ONE_HEX)?;
    let checksum_offset = checksum.len().checked_sub(64).ok_or("catalog too short")?;
    *checksum
        .get_mut(checksum_offset)
        .ok_or("catalog lacks checksum")? ^= 1;
    assert!(matches!(
        require_error(
            ChecksummedCatalog::decode(&checksum),
            "corrupt catalog checksum was admitted"
        )?,
        CatalogDecodeError::ChecksumMismatch { .. }
    ));

    let mut digest = catalog_bytes(GENERATION_ONE_HEX)?;
    let digest_offset = digest.len().checked_sub(32).ok_or("catalog too short")?;
    *digest
        .get_mut(digest_offset)
        .ok_or("catalog lacks digest")? ^= 1;
    assert!(matches!(
        require_error(
            ChecksummedCatalog::decode(&digest),
            "corrupt catalog digest was admitted"
        )?,
        CatalogDecodeError::DigestMismatch { .. }
    ));
    Ok(())
}
