//! Filesystem migration segment-pool admission laws.

use std::error::Error;

use super::StoreMigrationInventoryEntry;
use super::filesystem_inventory_segments;
use super::filesystem_inventory_segments_test_fixture::{
    SegmentPoolFixture, empty_bytes, maximum_policy, one_zero_bytes,
};
use crate::adapters::AdmittedSegment;

#[test]
fn every_canonical_segment_is_admitted_into_migration_inventory() -> Result<(), Box<dyn Error>> {
    let fixture = SegmentPoolFixture::create("migration-segment-inventory")?;
    let first_bytes = one_zero_bytes()?;
    let second_bytes = empty_bytes()?;
    let first = AdmittedSegment::decode(&first_bytes, maximum_policy())?;
    let second = AdmittedSegment::decode(&second_bytes, maximum_policy())?;
    let _first_digest = fixture.write_segment(&first_bytes)?;
    let _second_digest = fixture.write_segment(&second_bytes)?;

    let pool = fixture.open()?;
    let inventory = filesystem_inventory_segments::read(&pool, 2, maximum_policy())?;
    let mut expected = [
        StoreMigrationInventoryEntry::from_segment(&first),
        StoreMigrationInventoryEntry::from_segment(&second),
    ];
    expected.sort_unstable();

    assert_eq!(inventory.entries(), expected.as_slice());
    assert!(inventory.contains(first.digest()));
    assert!(inventory.contains(second.digest()));
    drop(pool);
    fixture.remove()?;
    Ok(())
}
