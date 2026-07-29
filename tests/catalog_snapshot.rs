//! Publication-head to immutable catalog snapshot laws.

#[path = "publication_head/format_oracle.rs"]
mod format_oracle;
mod support;

use std::error::Error;

use keep::{
    AdmittedCatalog, AdmittedSegment, CatalogSnapshotError, ChecksummedCatalog,
    ChecksummedPublicationHead, ChunkId, LayoutEntryLimit, SegmentReadPolicy,
    SegmentRecordIdentity, SegmentRecordLimit,
};
use support::{decode_hex, require_error};

const CATALOG_HEX: &str = include_str!("../conformance/segment-store/v1/one-zero-catalog.hex");
const HEAD_HEX: &str = include_str!("../conformance/segment-store/v1/one-zero-head.hex");
const GENERATION_TWO_HEAD_HEX: &str =
    include_str!("../conformance/segment-store/v1/one-zero-head-generation-two.hex");
const BUNDLE_HEAD_HEX: &str =
    include_str!("../conformance/segment-store/v1/one-zero-bundle-head.hex");
const SEGMENT_HEX: &str = include_str!("../conformance/segment-store/v1/one-zero-segment.hex");
const CATALOG_DIGEST_FIELD: usize = 40;

#[test]
fn snapshot_pins_one_complete_generation_across_later_head_reads() -> Result<(), Box<dyn Error>> {
    let catalog_bytes = fixture(CATALOG_HEX)?;
    let head_bytes = fixture(HEAD_HEX)?;
    let segment_bytes = fixture(SEGMENT_HEX)?;
    let catalog = admitted_catalog(&catalog_bytes, &segment_bytes)?;
    let head = ChecksummedPublicationHead::decode(&head_bytes)?;
    let snapshot = head.admit(catalog)?;

    let later_head_bytes = fixture(GENERATION_TWO_HEAD_HEX)?;
    let later_head = ChecksummedPublicationHead::decode(&later_head_bytes)?;
    let identity = SegmentRecordIdentity::Chunk(ChunkId::hash_bytes(&[0])?);

    assert_eq!(snapshot.generation().get(), 1);
    assert_eq!(later_head.generation().get(), 2);
    assert_eq!(
        snapshot
            .record(identity)
            .ok_or("snapshot omitted its pinned logical record")?
            .payload(),
        [0]
    );
    Ok(())
}

#[test]
fn snapshot_requires_the_head_generation_exactly() -> Result<(), Box<dyn Error>> {
    let catalog_bytes = fixture(CATALOG_HEX)?;
    let head_bytes = fixture(GENERATION_TWO_HEAD_HEX)?;
    let segment_bytes = fixture(SEGMENT_HEX)?;
    let catalog = admitted_catalog(&catalog_bytes, &segment_bytes)?;
    let head = ChecksummedPublicationHead::decode(&head_bytes)?;
    let error = require_error(
        head.admit(catalog),
        "generation-mismatched snapshot was admitted",
    )?;

    assert!(matches!(
        error,
        CatalogSnapshotError::Generation {
            expected,
            observed,
        } if expected.get() == 2 && observed.get() == 1
    ));
    Ok(())
}

#[test]
fn snapshot_requires_the_head_catalog_length_exactly() -> Result<(), Box<dyn Error>> {
    let catalog_bytes = fixture(CATALOG_HEX)?;
    let head_bytes = fixture(BUNDLE_HEAD_HEX)?;
    let segment_bytes = fixture(SEGMENT_HEX)?;
    let catalog = admitted_catalog(&catalog_bytes, &segment_bytes)?;
    let head = ChecksummedPublicationHead::decode(&head_bytes)?;
    let error = require_error(
        head.admit(catalog),
        "length-mismatched snapshot was admitted",
    )?;

    assert!(matches!(
        error,
        CatalogSnapshotError::CatalogLength {
            expected,
            observed,
        } if expected.get() == 512 && observed.get() == 352
    ));
    Ok(())
}

#[test]
fn snapshot_requires_the_head_catalog_digest_exactly() -> Result<(), Box<dyn Error>> {
    let catalog_bytes = fixture(CATALOG_HEX)?;
    let mut head_bytes = fixture(HEAD_HEX)?;
    *head_bytes
        .get_mut(CATALOG_DIGEST_FIELD)
        .ok_or("head fixture lacks catalog digest")? ^= 1;
    format_oracle::seal(&mut head_bytes)?;
    let segment_bytes = fixture(SEGMENT_HEX)?;
    let catalog = admitted_catalog(&catalog_bytes, &segment_bytes)?;
    let head = ChecksummedPublicationHead::decode(&head_bytes)?;
    let expected = head.catalog_digest();
    let observed = catalog.digest();
    let error = require_error(
        head.admit(catalog),
        "digest-mismatched snapshot was admitted",
    )?;

    assert!(matches!(
        error,
        CatalogSnapshotError::CatalogDigest {
            expected: error_expected,
            observed: error_observed,
        } if error_expected == expected && error_observed == observed
    ));
    Ok(())
}

fn admitted_catalog<'catalog, 'segment>(
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

fn fixture(hex: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    decode_hex(hex.strip_suffix('\n').ok_or("fixture must end in one LF")?).map_err(Into::into)
}
