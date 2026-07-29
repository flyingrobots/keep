//! Catalog-to-segment top-level record binding laws.

#[path = "catalog/format_oracle.rs"]
mod format_oracle;
#[path = "catalog_locations/refusal_laws.rs"]
mod refusal_laws;
mod support;

use std::error::Error;

use keep::{
    AdmittedSegment, CatalogAdmissionError, ChecksummedCatalog, ChunkId, LayoutEntryLimit,
    SegmentReadPolicy, SegmentRecordIdentity, SegmentRecordLimit,
};
use support::{decode_hex, require_error};

const CATALOG_HEX: &str = include_str!("../conformance/segment-store/v1/one-zero-catalog.hex");
const SEGMENT_HEX: &str = include_str!("../conformance/segment-store/v1/one-zero-segment.hex");
const BUNDLE_CATALOG_HEX: &str =
    include_str!("../conformance/segment-store/v1/one-zero-bundle-catalog.hex");
const BUNDLE_SEGMENT_HEX: &str =
    include_str!("../conformance/segment-store/v1/one-zero-bundle-segment.hex");
const RECORD_OFFSET_FIELD: usize = 128 + 96;
const ENTRY_IDENTITY_DIGEST_FIELD: usize = 128 + 8;
const ENTRY_SEGMENT_DIGEST_FIELD: usize = 128 + 64;
const ENTRY_CHECKSUM_FIELD: usize = 128 + 120;

#[test]
fn admitted_catalog_resolves_logical_records_without_exposing_locations()
-> Result<(), Box<dyn Error>> {
    let catalog_bytes = fixture(CATALOG_HEX)?;
    let segment_bytes = fixture(SEGMENT_HEX)?;
    let catalog = ChecksummedCatalog::decode(&catalog_bytes)?;
    let segment = AdmittedSegment::decode(&segment_bytes, maximum_policy())?;
    let admitted = catalog.admit(&[segment])?;
    let identity = SegmentRecordIdentity::Chunk(ChunkId::hash_bytes(&[0])?);
    let record = admitted
        .record(identity)
        .ok_or("admitted catalog omitted its logical record")?;

    assert_eq!(admitted.generation().get(), 1);
    assert_eq!(admitted.record_count(), 1);
    assert_eq!(record.identity(), identity);
    assert_eq!(record.payload(), [0]);
    Ok(())
}

#[test]
fn admitted_bundle_resolves_every_catalog_identity() -> Result<(), Box<dyn Error>> {
    let catalog_bytes = fixture(BUNDLE_CATALOG_HEX)?;
    let segment_bytes = fixture(BUNDLE_SEGMENT_HEX)?;
    let catalog = ChecksummedCatalog::decode(&catalog_bytes)?;
    let segment = AdmittedSegment::decode(&segment_bytes, maximum_policy())?;
    let expected = segment
        .records()
        .map(|record| record.map(keep::AdmittedSegmentRecord::identity))
        .collect::<Result<Vec<_>, _>>()?;
    let admitted = catalog.admit(&[segment])?;

    assert_eq!(admitted.record_count(), 2);
    for identity in expected {
        assert_eq!(
            admitted
                .record(identity)
                .ok_or("bundle catalog omitted a logical record")?
                .identity(),
            identity
        );
    }
    Ok(())
}

#[test]
fn catalog_requires_the_exact_named_admitted_segment() -> Result<(), Box<dyn Error>> {
    let catalog_bytes = fixture(CATALOG_HEX)?;
    let catalog = ChecksummedCatalog::decode(&catalog_bytes)?;
    let error = require_error(
        catalog.admit(&[]),
        "catalog without its named segment was admitted",
    )?;

    assert!(matches!(
        error,
        CatalogAdmissionError::MissingSegment { .. }
    ));
    Ok(())
}

#[test]
fn catalog_location_must_equal_one_discovered_top_level_record_span() -> Result<(), Box<dyn Error>>
{
    let mut catalog_bytes = fixture(CATALOG_HEX)?;
    replace_u64(&mut catalog_bytes, RECORD_OFFSET_FIELD, 65)?;
    format_oracle::seal(&mut catalog_bytes)?;
    let catalog = ChecksummedCatalog::decode(&catalog_bytes)?;
    let segment_bytes = fixture(SEGMENT_HEX)?;
    let segment = AdmittedSegment::decode(&segment_bytes, maximum_policy())?;
    let error = require_error(
        catalog.admit(&[segment]),
        "interior record location was admitted",
    )?;

    assert!(matches!(
        error,
        CatalogAdmissionError::LocationNotTopLevel {
            record_offset: 65,
            record_length: 145,
            ..
        }
    ));
    Ok(())
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
