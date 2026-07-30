//! Canonical version-2 store-migration intent encoding laws.

#[path = "store_migration_intent/fixture.rs"]
mod fixture;
mod support;

use std::error::Error;

use keep::{
    AdmittedSegment, AdmittedStoreMigrationIntent, CanonicalStoreMigrationIntent,
    ChecksummedCatalog, ChecksummedPublicationHead, LayoutEntryLimit, SegmentReadPolicy,
    SegmentRecordLimit,
};
use support::decode_hex;

const SEGMENT: &str = include_str!("../conformance/segment-store/v1/one-zero-segment.hex");
const CATALOG: &str = include_str!("../conformance/segment-store/v1/one-zero-catalog.hex");
const HEAD: &str = include_str!("../conformance/segment-store/v1/one-zero-head.hex");
const CATALOG_TWO: &str =
    include_str!("../conformance/segment-store/v1/one-zero-catalog-generation-two.hex");
const HEAD_TWO: &str =
    include_str!("../conformance/segment-store/v1/one-zero-head-generation-two.hex");
const PREDECESSOR_OFFSET: usize = 72;
const PREDECESSOR_END: usize = 104;

#[test]
fn admitted_coordinates_reproduce_the_frozen_intent() -> Result<(), Box<dyn Error>> {
    let expected = fixture::fixture_bytes()?;
    let admitted = AdmittedStoreMigrationIntent::decode(&expected)?;
    assert_eq!(
        admitted.inventory_digest().as_bytes(),
        &fixture::INVENTORY_DIGEST
    );
    let canonical = canonical_intent(CATALOG, HEAD, &admitted)?;

    assert_eq!(canonical.encoded(), expected);
    assert_eq!(canonical.digest(), admitted.digest());
    assert_eq!(canonical.store_identifier(), admitted.store_identifier());
    assert_eq!(canonical.digest().as_bytes(), &fixture::INTENT_DIGEST);
    assert_eq!(
        canonical.store_identifier().as_bytes(),
        &fixture::STORE_IDENTIFIER
    );
    Ok(())
}

#[test]
fn successor_intent_encodes_the_exact_predecessor() -> Result<(), Box<dyn Error>> {
    let source_bytes = fixture::fixture_bytes()?;
    let source = AdmittedStoreMigrationIntent::decode(&source_bytes)?;
    let canonical = canonical_intent(CATALOG_TWO, HEAD_TWO, &source)?;
    let admitted = AdmittedStoreMigrationIntent::decode(canonical.encoded())?;

    assert_eq!(admitted.catalog_generation().get(), 2);
    assert_eq!(
        canonical.encoded().get(PREDECESSOR_OFFSET..PREDECESSOR_END),
        Some(fixture::CATALOG_DIGEST.as_slice())
    );
    assert_eq!(
        admitted
            .predecessor_catalog_digest()
            .ok_or("successor intent omitted its predecessor")?
            .as_bytes(),
        &fixture::CATALOG_DIGEST
    );
    Ok(())
}

fn canonical_intent(
    catalog_hex: &str,
    head_hex: &str,
    source: &AdmittedStoreMigrationIntent<'_>,
) -> Result<CanonicalStoreMigrationIntent, Box<dyn Error>> {
    let segment_bytes = protocol_fixture(SEGMENT)?;
    let catalog_bytes = protocol_fixture(catalog_hex)?;
    let head_bytes = protocol_fixture(head_hex)?;
    let segment = AdmittedSegment::decode(&segment_bytes, maximum_policy())?;
    let catalog = ChecksummedCatalog::decode(&catalog_bytes)?.admit(&[segment])?;
    let snapshot = ChecksummedPublicationHead::decode(&head_bytes)?.admit(catalog)?;
    Ok(CanonicalStoreMigrationIntent::from_snapshot(
        &snapshot,
        source.inventory_digest(),
        source.root_device_identity(),
        source.root_mount_identity(),
        source.root_file_identity(),
    ))
}

const fn maximum_policy() -> SegmentReadPolicy {
    SegmentReadPolicy::new(SegmentRecordLimit::MAXIMUM, LayoutEntryLimit::MAXIMUM)
}

fn protocol_fixture(hex: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    decode_hex(hex.strip_suffix('\n').ok_or("fixture must end in one LF")?).map_err(Into::into)
}
