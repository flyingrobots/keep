//! Canonical catalog and publication-head emission laws.

mod support;

use std::error::Error;

use keep::{
    AdmittedSegment, CanonicalCatalog, CanonicalPublicationHead, CatalogEncodeError,
    CatalogGeneration, ChecksummedCatalog, ChunkId, LayoutEntryLimit, SegmentReadPolicy,
    SegmentRecordIdentity, SegmentRecordLimit,
};
use support::{decode_hex, require_error};

const ONE_ZERO_SEGMENT_HEX: &str =
    include_str!("../conformance/segment-store/v1/one-zero-segment.hex");
const EMPTY_SEGMENT_HEX: &str = include_str!("../conformance/segment-store/v1/empty-segment.hex");
const BUNDLE_SEGMENT_HEX: &str =
    include_str!("../conformance/segment-store/v1/one-zero-bundle-segment.hex");
const ONE_ZERO_CATALOG_HEX: &str =
    include_str!("../conformance/segment-store/v1/one-zero-catalog.hex");
const GENERATION_TWO_CATALOG_HEX: &str =
    include_str!("../conformance/segment-store/v1/one-zero-catalog-generation-two.hex");
const BUNDLE_CATALOG_HEX: &str =
    include_str!("../conformance/segment-store/v1/one-zero-bundle-catalog.hex");
const ONE_ZERO_HEAD_HEX: &str = include_str!("../conformance/segment-store/v1/one-zero-head.hex");
const GENERATION_TWO_HEAD_HEX: &str =
    include_str!("../conformance/segment-store/v1/one-zero-head-generation-two.hex");
const BUNDLE_HEAD_HEX: &str =
    include_str!("../conformance/segment-store/v1/one-zero-bundle-head.hex");

#[test]
fn admitted_segments_reproduce_every_frozen_catalog() -> Result<(), Box<dyn Error>> {
    let one_zero_bytes = fixture(ONE_ZERO_SEGMENT_HEX)?;
    let bundle_bytes = fixture(BUNDLE_SEGMENT_HEX)?;
    let one_zero = admitted_segment(&one_zero_bytes)?;
    let bundle = admitted_segment(&bundle_bytes)?;
    let one_zero_segments = [one_zero];
    let bundle_segments = [bundle];

    let first = CanonicalCatalog::from_segments(generation(1)?, None, &one_zero_segments)?;
    let second = CanonicalCatalog::from_segments(
        generation(2)?,
        Some(first.checksummed().digest()),
        &one_zero_segments,
    )?;
    let bundled = CanonicalCatalog::from_segments(generation(1)?, None, &bundle_segments)?;

    assert_eq!(first.encoded(), fixture(ONE_ZERO_CATALOG_HEX)?);
    assert_eq!(second.encoded(), fixture(GENERATION_TWO_CATALOG_HEX)?);
    assert_eq!(bundled.encoded(), fixture(BUNDLE_CATALOG_HEX)?);
    Ok(())
}

#[test]
fn checksummed_catalogs_reproduce_every_frozen_head() -> Result<(), Box<dyn Error>> {
    assert_head(ONE_ZERO_CATALOG_HEX, ONE_ZERO_HEAD_HEX)?;
    assert_head(GENERATION_TWO_CATALOG_HEX, GENERATION_TWO_HEAD_HEX)?;
    assert_head(BUNDLE_CATALOG_HEX, BUNDLE_HEAD_HEX)
}

#[test]
fn every_generation_enforces_its_exact_predecessor_law() -> Result<(), Box<dyn Error>> {
    let segment_bytes = fixture(ONE_ZERO_SEGMENT_HEX)?;
    let segments = [admitted_segment(&segment_bytes)?];
    let first = CanonicalCatalog::from_segments(generation(1)?, None, &segments)?;
    let predecessor = first.checksummed().digest();

    let missing = require_error(
        CanonicalCatalog::from_segments(generation(2)?, None, &segments),
        "later catalog omitted its predecessor",
    )?;
    let unexpected = require_error(
        CanonicalCatalog::from_segments(generation(1)?, Some(predecessor), &segments),
        "generation 1 admitted a predecessor",
    )?;

    assert!(matches!(
        missing,
        CatalogEncodeError::MissingPredecessor { generation } if generation.get() == 2
    ));
    assert!(matches!(
        unexpected,
        CatalogEncodeError::UnexpectedPredecessor { observed } if observed == predecessor
    ));
    Ok(())
}

#[test]
fn duplicate_logical_records_are_refused_before_emission() -> Result<(), Box<dyn Error>> {
    let first_bytes = fixture(ONE_ZERO_SEGMENT_HEX)?;
    let second_bytes = fixture(ONE_ZERO_SEGMENT_HEX)?;
    let segments = [
        admitted_segment(&first_bytes)?,
        admitted_segment(&second_bytes)?,
    ];
    let expected = SegmentRecordIdentity::Chunk(ChunkId::hash_bytes(&[0])?);
    let error = require_error(
        CanonicalCatalog::from_segments(generation(1)?, None, &segments),
        "duplicate logical records were encoded",
    )?;

    assert!(matches!(
        error,
        CatalogEncodeError::DuplicateIdentity { identity } if identity == expected
    ));
    Ok(())
}

#[test]
fn catalog_encoding_refuses_an_unreferenced_empty_segment() -> Result<(), Box<dyn Error>> {
    let empty_bytes = fixture(EMPTY_SEGMENT_HEX)?;
    let segments = [admitted_segment(&empty_bytes)?];
    let expected = segments[0].digest();
    let error = require_error(
        CanonicalCatalog::from_segments(generation(1)?, None, &segments),
        "empty physical segment was silently omitted from the catalog",
    )?;
    assert!(matches!(
        error,
        CatalogEncodeError::UnreferencedSegment { segment_digest }
            if segment_digest == expected
    ));
    Ok(())
}

fn assert_head(catalog_hex: &str, head_hex: &str) -> Result<(), Box<dyn Error>> {
    let catalog_bytes = fixture(catalog_hex)?;
    let catalog = ChecksummedCatalog::decode(&catalog_bytes)?;
    let head = CanonicalPublicationHead::for_catalog(catalog);
    assert_eq!(head.encoded().as_slice(), fixture(head_hex)?);
    Ok(())
}

fn admitted_segment(bytes: &[u8]) -> Result<AdmittedSegment<'_>, Box<dyn Error>> {
    AdmittedSegment::decode(bytes, maximum_policy()).map_err(Into::into)
}

const fn maximum_policy() -> SegmentReadPolicy {
    SegmentReadPolicy::new(SegmentRecordLimit::MAXIMUM, LayoutEntryLimit::MAXIMUM)
}

fn generation(value: u64) -> Result<CatalogGeneration, Box<dyn Error>> {
    CatalogGeneration::new(value).map_err(Into::into)
}

fn fixture(hex: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    decode_hex(hex.strip_suffix('\n').ok_or("fixture must end in one LF")?).map_err(Into::into)
}
