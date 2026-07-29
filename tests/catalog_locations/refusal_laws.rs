//! Catalog location-binding disagreement laws.

use std::error::Error;

use keep::{AdmittedSegment, CatalogAdmissionError, ChecksummedCatalog};

use super::{
    BUNDLE_CATALOG_HEX, BUNDLE_SEGMENT_HEX, CATALOG_HEX, ENTRY_CHECKSUM_FIELD,
    ENTRY_IDENTITY_DIGEST_FIELD, ENTRY_SEGMENT_DIGEST_FIELD, SEGMENT_HEX, fixture, format_oracle,
    maximum_policy,
};
use crate::support::require_error;

#[test]
fn catalog_identity_must_equal_the_selected_record_identity() -> Result<(), Box<dyn Error>> {
    let mut encoded = fixture(CATALOG_HEX)?;
    replace_byte(&mut encoded, ENTRY_IDENTITY_DIGEST_FIELD)?;
    format_oracle::seal(&mut encoded)?;
    let catalog = ChecksummedCatalog::decode(&encoded)?;
    let segment_bytes = fixture(SEGMENT_HEX)?;
    let segment = AdmittedSegment::decode(&segment_bytes, maximum_policy())?;
    let error = require_error(
        catalog.admit(&[segment]),
        "catalog identity disagreement was admitted",
    )?;

    assert!(matches!(
        error,
        CatalogAdmissionError::RecordIdentityMismatch { .. }
    ));
    Ok(())
}

#[test]
fn catalog_checksum_must_equal_the_selected_record_checksum() -> Result<(), Box<dyn Error>> {
    let mut encoded = fixture(CATALOG_HEX)?;
    replace_byte(&mut encoded, ENTRY_CHECKSUM_FIELD)?;
    format_oracle::seal(&mut encoded)?;
    let catalog = ChecksummedCatalog::decode(&encoded)?;
    let segment_bytes = fixture(SEGMENT_HEX)?;
    let segment = AdmittedSegment::decode(&segment_bytes, maximum_policy())?;
    let error = require_error(
        catalog.admit(&[segment]),
        "catalog checksum disagreement was admitted",
    )?;

    assert!(matches!(
        error,
        CatalogAdmissionError::RecordChecksumMismatch { .. }
    ));
    Ok(())
}

#[test]
fn catalog_segment_digest_selects_the_exact_physical_segment() -> Result<(), Box<dyn Error>> {
    let mut encoded = fixture(CATALOG_HEX)?;
    replace_byte(&mut encoded, ENTRY_SEGMENT_DIGEST_FIELD)?;
    format_oracle::seal(&mut encoded)?;
    let catalog = ChecksummedCatalog::decode(&encoded)?;
    let segment_bytes = fixture(SEGMENT_HEX)?;
    let segment = AdmittedSegment::decode(&segment_bytes, maximum_policy())?;
    let error = require_error(
        catalog.admit(&[segment]),
        "wrong physical segment digest was admitted",
    )?;

    assert!(matches!(
        error,
        CatalogAdmissionError::MissingSegment { .. }
    ));
    Ok(())
}

#[test]
fn segment_input_is_bounded_and_duplicate_free() -> Result<(), Box<dyn Error>> {
    let catalog_bytes = fixture(CATALOG_HEX)?;
    let segment_bytes = fixture(SEGMENT_HEX)?;
    let catalog = ChecksummedCatalog::decode(&catalog_bytes)?;
    let first = AdmittedSegment::decode(&segment_bytes, maximum_policy())?;
    let second = AdmittedSegment::decode(&segment_bytes, maximum_policy())?;
    let error = require_error(
        catalog.admit(&[first, second]),
        "excess segment input was admitted",
    )?;
    assert!(matches!(
        error,
        CatalogAdmissionError::SegmentCountOutOfBounds {
            maximum: 1,
            observed: 2,
        }
    ));

    let bundle_catalog_bytes = fixture(BUNDLE_CATALOG_HEX)?;
    let bundle_segment_bytes = fixture(BUNDLE_SEGMENT_HEX)?;
    let bundle_catalog = ChecksummedCatalog::decode(&bundle_catalog_bytes)?;
    let first = AdmittedSegment::decode(&bundle_segment_bytes, maximum_policy())?;
    let second = AdmittedSegment::decode(&bundle_segment_bytes, maximum_policy())?;
    let error = require_error(
        bundle_catalog.admit(&[first, second]),
        "duplicate physical segment input was admitted",
    )?;
    assert!(matches!(
        error,
        CatalogAdmissionError::DuplicateSegment { .. }
    ));
    Ok(())
}

fn replace_byte(target: &mut [u8], offset: usize) -> Result<(), Box<dyn Error>> {
    let byte = target
        .get_mut(offset)
        .ok_or("catalog fixture lacks mutation byte")?;
    *byte ^= 1;
    Ok(())
}
