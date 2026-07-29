//! Catalog-stage truncation and complete-admission laws.

use std::error::Error;

use keep::{
    CatalogDecodeError, RecoveryCatalogStage, RecoveryCatalogStageError,
    classify_recovery_catalog_stage,
};

use super::{CATALOG_HEADER_LENGTH, CATALOG_HEX, fixture};

#[test]
fn canonical_catalog_stage_is_complete() -> Result<(), Box<dyn Error>> {
    let encoded = fixture(CATALOG_HEX)?;

    let RecoveryCatalogStage::Complete(catalog) = classify_recovery_catalog_stage(&encoded)? else {
        return Err("canonical catalog stage was not complete".into());
    };

    assert_eq!(catalog.encoded(), encoded);
    assert_eq!(catalog.generation().get(), 1);
    Ok(())
}

#[test]
fn partial_catalog_header_is_exactly_truncated() -> Result<(), Box<dyn Error>> {
    let complete = fixture(CATALOG_HEX)?;
    let observed = CATALOG_HEADER_LENGTH
        .checked_sub(1)
        .ok_or("catalog header underflow")?;
    let encoded = complete
        .get(..observed)
        .ok_or("missing partial catalog header")?;

    let state = classify_recovery_catalog_stage(encoded)?;

    assert!(matches!(
        state,
        RecoveryCatalogStage::HeaderTruncated {
            required: CATALOG_HEADER_LENGTH,
            observed: actual,
        } if actual == observed
    ));
    Ok(())
}

#[test]
fn partial_declared_catalog_body_is_exactly_truncated() -> Result<(), Box<dyn Error>> {
    let complete = fixture(CATALOG_HEX)?;
    let observed = complete.len().checked_sub(1).ok_or("catalog underflow")?;
    let encoded = complete.get(..observed).ok_or("missing partial catalog")?;

    let state = classify_recovery_catalog_stage(encoded)?;

    assert!(matches!(
        state,
        RecoveryCatalogStage::BodyTruncated {
            expected: 352,
            observed: actual,
        } if actual == observed
    ));
    Ok(())
}

#[test]
fn complete_invalid_catalog_header_is_a_header_refusal() -> Result<(), Box<dyn Error>> {
    let mut encoded = fixture(CATALOG_HEX)?;
    let byte = encoded.first_mut().ok_or("missing catalog header")?;
    *byte ^= 1;

    let error = classify_recovery_catalog_stage(&encoded)
        .err()
        .ok_or("invalid catalog header was classified as lawful")?;

    assert!(matches!(
        error,
        RecoveryCatalogStageError::Header {
            source: CatalogDecodeError::InvalidMagic { .. },
        }
    ));
    Ok(())
}

#[test]
fn complete_invalid_catalog_checksum_is_a_complete_refusal() -> Result<(), Box<dyn Error>> {
    let mut encoded = fixture(CATALOG_HEX)?;
    let byte = encoded.last_mut().ok_or("missing catalog checksum")?;
    *byte ^= 1;

    let error = classify_recovery_catalog_stage(&encoded)
        .err()
        .ok_or("invalid catalog checksum was classified as lawful")?;

    assert!(matches!(
        error,
        RecoveryCatalogStageError::Complete {
            source: CatalogDecodeError::DigestMismatch { .. }
                | CatalogDecodeError::ChecksumMismatch { .. },
        }
    ));
    Ok(())
}
