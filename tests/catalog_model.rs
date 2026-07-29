//! Deterministic catalog transition and lookup model laws.

mod support;

use std::collections::BTreeMap;
use std::error::Error;

use keep::{
    AdmittedSegment, CanonicalCatalog, CanonicalPublicationHead, CatalogGeneration,
    ChecksummedPublicationHead, LayoutEntryLimit, SegmentReadPolicy, SegmentRecordIdentity,
    SegmentRecordLimit,
};
use support::decode_hex;

type ReferenceCatalog = BTreeMap<SegmentRecordIdentity, Vec<u8>>;

const BUNDLE_SEGMENT_HEX: &str =
    include_str!("../conformance/segment-store/v1/one-zero-bundle-segment.hex");
const ONE_SEGMENT_HEX: &str = include_str!("../conformance/segment-store/v1/one-zero-segment.hex");

#[test]
fn generation_transitions_and_lookups_match_a_btree_map() -> Result<(), Box<dyn Error>> {
    let bundle_bytes = fixture(BUNDLE_SEGMENT_HEX)?;
    let one_bytes = fixture(ONE_SEGMENT_HEX)?;
    let bundle_segments = [AdmittedSegment::decode(&bundle_bytes, policy())?];
    let one_segments = [AdmittedSegment::decode(&one_bytes, policy())?];
    let empty_segments: [AdmittedSegment<'_>; 0] = [];

    let first =
        CanonicalCatalog::from_segments(CatalogGeneration::new(1)?, None, &bundle_segments)?;
    let second = CanonicalCatalog::from_segments(
        CatalogGeneration::new(2)?,
        Some(first.checksummed().digest()),
        &one_segments,
    )?;
    let third = CanonicalCatalog::from_segments(
        CatalogGeneration::new(3)?,
        Some(second.checksummed().digest()),
        &empty_segments,
    )?;

    assert_snapshot_matches(&first, &bundle_segments, &model(&bundle_segments)?)?;
    assert_snapshot_matches(&second, &one_segments, &model(&one_segments)?)?;
    assert_snapshot_matches(&third, &empty_segments, &BTreeMap::new())?;
    assert_successor(&first, &bundle_segments, &second, &one_segments, 2)?;
    assert_successor(&second, &one_segments, &third, &empty_segments, 3)
}

fn assert_snapshot_matches(
    catalog: &CanonicalCatalog,
    segments: &[AdmittedSegment<'_>],
    expected: &ReferenceCatalog,
) -> Result<(), Box<dyn Error>> {
    let checked = catalog.checksummed();
    let admitted = checked.admit(segments)?;
    let head = CanonicalPublicationHead::for_catalog(checked);
    let snapshot = ChecksummedPublicationHead::decode(head.encoded())?.admit(admitted)?;

    assert_eq!(snapshot.record_count(), u64::try_from(expected.len())?);
    for (identity, payload) in expected {
        assert_eq!(
            snapshot
                .record(*identity)
                .ok_or("model identity missing from snapshot")?
                .payload(),
            payload
        );
    }
    Ok(())
}

fn assert_successor(
    current: &CanonicalCatalog,
    current_segments: &[AdmittedSegment<'_>],
    candidate: &CanonicalCatalog,
    candidate_segments: &[AdmittedSegment<'_>],
    expected_generation: u64,
) -> Result<(), Box<dyn Error>> {
    let current = current.checksummed().admit(current_segments)?;
    let candidate = candidate.checksummed().admit(candidate_segments)?;
    let successor = current.validate_successor(candidate)?;
    assert_eq!(successor.generation().get(), expected_generation);
    Ok(())
}

fn model(segments: &[AdmittedSegment<'_>]) -> Result<ReferenceCatalog, Box<dyn Error>> {
    let mut model = BTreeMap::new();
    for segment in segments {
        for record in segment.records() {
            let record = record?;
            if model
                .insert(record.identity(), record.payload().to_vec())
                .is_some()
            {
                return Err("model input contains a duplicate identity".into());
            }
        }
    }
    Ok(model)
}

const fn policy() -> SegmentReadPolicy {
    SegmentReadPolicy::new(SegmentRecordLimit::MAXIMUM, LayoutEntryLimit::MAXIMUM)
}

fn fixture(hex: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    decode_hex(hex.strip_suffix('\n').ok_or("fixture must end in one LF")?).map_err(Into::into)
}
