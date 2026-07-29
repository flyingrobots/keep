//! Exact catalog successor admission laws.

#[path = "catalog/format_oracle.rs"]
mod format_oracle;
mod support;

use std::error::Error;

use keep::{
    AdmittedCatalog, AdmittedSegment, CatalogGenerationError, CatalogTransitionError,
    ChecksummedCatalog, LayoutEntryLimit, SegmentReadPolicy, SegmentRecordLimit,
};
use support::{decode_hex, require_error};

const GENERATION_ONE_HEX: &str =
    include_str!("../conformance/segment-store/v1/one-zero-catalog.hex");
const GENERATION_TWO_HEX: &str =
    include_str!("../conformance/segment-store/v1/one-zero-catalog-generation-two.hex");
const SEGMENT_HEX: &str = include_str!("../conformance/segment-store/v1/one-zero-segment.hex");
const GENERATION_FIELD: usize = 24;
const PREDECESSOR_FIELD: usize = 32;

#[test]
fn exact_successor_preserves_the_admitted_candidate() -> Result<(), Box<dyn Error>> {
    let segment_bytes = fixture(SEGMENT_HEX)?;
    let current_bytes = fixture(GENERATION_ONE_HEX)?;
    let candidate_bytes = fixture(GENERATION_TWO_HEX)?;
    let current = admitted(&current_bytes, &segment_bytes)?;
    let candidate = admitted(&candidate_bytes, &segment_bytes)?;
    let successor = current.validate_successor(candidate)?;

    assert_eq!(successor.generation().get(), 2);
    assert_eq!(successor.catalog().record_count(), 1);
    Ok(())
}

#[test]
fn stale_generation_reports_expected_and_observed_coordinates() -> Result<(), Box<dyn Error>> {
    let segment_bytes = fixture(SEGMENT_HEX)?;
    let current_bytes = fixture(GENERATION_ONE_HEX)?;
    let stale_bytes = fixture(GENERATION_ONE_HEX)?;
    let current = admitted(&current_bytes, &segment_bytes)?;
    let stale = admitted(&stale_bytes, &segment_bytes)?;
    let error = require_error(
        current.validate_successor(stale),
        "stale generation was admitted as a successor",
    )?;

    assert!(matches!(
        error,
        CatalogTransitionError::Generation {
            expected,
            observed,
        } if expected.get() == 2 && observed.get() == 1
    ));
    Ok(())
}

#[test]
fn wrong_predecessor_reports_expected_and_observed_digests() -> Result<(), Box<dyn Error>> {
    let segment_bytes = fixture(SEGMENT_HEX)?;
    let current_bytes = fixture(GENERATION_ONE_HEX)?;
    let mut candidate_bytes = fixture(GENERATION_TWO_HEX)?;
    *candidate_bytes
        .get_mut(PREDECESSOR_FIELD)
        .ok_or("candidate lacks predecessor digest")? ^= 1;
    format_oracle::seal(&mut candidate_bytes)?;
    let current = admitted(&current_bytes, &segment_bytes)?;
    let expected = current.digest();
    let candidate = admitted(&candidate_bytes, &segment_bytes)?;
    let observed = candidate
        .previous_catalog_digest()
        .ok_or("mutated candidate omitted its predecessor")?;
    let error = require_error(
        current.validate_successor(candidate),
        "wrong predecessor was admitted as a successor",
    )?;

    assert!(matches!(
        error,
        CatalogTransitionError::Predecessor {
            expected: error_expected,
            observed: Some(error_observed),
        } if error_expected == expected && error_observed == observed
    ));
    Ok(())
}

#[test]
fn maximum_generation_refuses_successor_derivation() -> Result<(), Box<dyn Error>> {
    let segment_bytes = fixture(SEGMENT_HEX)?;
    let mut current_bytes = fixture(GENERATION_TWO_HEX)?;
    replace_u64(&mut current_bytes, GENERATION_FIELD, u64::MAX)?;
    format_oracle::seal(&mut current_bytes)?;
    let candidate_bytes = fixture(GENERATION_TWO_HEX)?;
    let current = admitted(&current_bytes, &segment_bytes)?;
    let candidate = admitted(&candidate_bytes, &segment_bytes)?;
    let error = require_error(
        current.validate_successor(candidate),
        "successor was derived after generation exhaustion",
    )?;

    assert!(matches!(
        error,
        CatalogTransitionError::GenerationExhausted {
            source: CatalogGenerationError::Exhausted { current: u64::MAX },
        }
    ));
    Ok(())
}

fn admitted<'catalog, 'segment>(
    catalog_bytes: &'catalog [u8],
    segment_bytes: &'segment [u8],
) -> Result<AdmittedCatalog<'catalog, 'segment>, Box<dyn Error>> {
    let catalog = ChecksummedCatalog::decode(catalog_bytes)?;
    let segment = AdmittedSegment::decode(segment_bytes, maximum_policy())?;
    catalog.admit(&[segment]).map_err(Into::into)
}

const fn maximum_policy() -> SegmentReadPolicy {
    SegmentReadPolicy::new(SegmentRecordLimit::MAXIMUM, LayoutEntryLimit::MAXIMUM)
}

fn replace_u64(target: &mut [u8], offset: usize, value: u64) -> Result<(), Box<dyn Error>> {
    let end = offset.checked_add(8).ok_or("test offset overflow")?;
    target
        .get_mut(offset..end)
        .ok_or("catalog fixture lacks u64 field")?
        .copy_from_slice(&value.to_be_bytes());
    Ok(())
}

fn fixture(hex: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    decode_hex(hex.strip_suffix('\n').ok_or("fixture must end in one LF")?).map_err(Into::into)
}
