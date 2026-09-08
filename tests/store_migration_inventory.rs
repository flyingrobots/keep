//! Canonical version-1 immutable-pool migration inventory laws.

mod support;

use std::error::Error;

use keep::{
    AdmittedSegment, ChecksummedCatalog, LayoutEntryLimit, SegmentReadPolicy, SegmentRecordLimit,
    StoreMigrationInventoryEntry, StoreMigrationInventoryEntryCount,
    StoreMigrationInventoryEntryCountError, StoreMigrationInventoryError,
    StoreMigrationInventoryHasher,
};
use support::decode_hex;

const SEGMENT: &str = include_str!("../conformance/segment-store/v1/one-zero-segment.hex");
const CATALOG: &str = include_str!("../conformance/segment-store/v1/one-zero-catalog.hex");
const SEGMENT_ENTRY: &str = concat!(
    "0100000000000000",
    "0000000000000000",
    "0000000000000151",
    "b7542dced2ab770894a14d1d04b066e3a899942602c5986d35ba6df6c1a35cfc",
);
const CATALOG_ENTRY: &str = concat!(
    "0200000000000000",
    "0000000000000001",
    "0000000000000160",
    "04b82519b0399baefd0b9c0f32a871052e4c47e3a00226ab03b21661470f7320",
);
const INVENTORY_DIGEST: &str = "40bf5d49c34847ac9cf46a256f343cee80cd980d1405d2dd02ceff8f58d674f9";

#[test]
fn frozen_inventory_entries_and_digest_are_exact() -> Result<(), Box<dyn Error>> {
    let (segment, catalog) = frozen_entries()?;
    assert_eq!(segment.encoded().as_slice(), decode_hex(SEGMENT_ENTRY)?);
    assert_eq!(catalog.encoded().as_slice(), decode_hex(CATALOG_ENTRY)?);

    let count = StoreMigrationInventoryEntryCount::new(2)?;
    let mut inventory = StoreMigrationInventoryHasher::new(count);
    inventory.push(segment)?;
    inventory.push(catalog)?;
    let digest = inventory.finish()?;
    assert_eq!(digest.as_bytes().as_slice(), decode_hex(INVENTORY_DIGEST)?);
    Ok(())
}

#[test]
fn inventory_refuses_duplicate_and_out_of_order_entries() -> Result<(), Box<dyn Error>> {
    let (segment, catalog) = frozen_entries()?;
    let count = StoreMigrationInventoryEntryCount::new(2)?;

    let mut duplicate = StoreMigrationInventoryHasher::new(count);
    duplicate.push(segment)?;
    assert_eq!(
        duplicate.push(segment),
        Err(StoreMigrationInventoryError::Duplicate { entry: segment })
    );

    let mut out_of_order = StoreMigrationInventoryHasher::new(count);
    out_of_order.push(catalog)?;
    assert_eq!(
        out_of_order.push(segment),
        Err(StoreMigrationInventoryError::OutOfOrder {
            previous: catalog,
            observed: segment,
        })
    );
    Ok(())
}

#[test]
fn inventory_refuses_count_overrun_and_incomplete_finalization() -> Result<(), Box<dyn Error>> {
    let (segment, catalog) = frozen_entries()?;
    let one = StoreMigrationInventoryEntryCount::new(1)?;
    let two = StoreMigrationInventoryEntryCount::new(2)?;

    let mut overrun = StoreMigrationInventoryHasher::new(one);
    overrun.push(segment)?;
    assert_eq!(
        overrun.push(catalog),
        Err(StoreMigrationInventoryError::EntryCountExceeded {
            expected: one,
            observed: 2,
        })
    );

    let mut incomplete = StoreMigrationInventoryHasher::new(two);
    incomplete.push(segment)?;
    assert_eq!(
        incomplete.finish(),
        Err(StoreMigrationInventoryError::Incomplete {
            expected: two,
            observed: 1,
        })
    );
    Ok(())
}

#[test]
fn inventory_count_has_the_exact_protocol_bound() {
    assert_eq!(
        StoreMigrationInventoryEntryCount::new(0).map(StoreMigrationInventoryEntryCount::get),
        Ok(0)
    );
    assert_eq!(
        StoreMigrationInventoryEntryCount::new(StoreMigrationInventoryEntryCount::MAXIMUM)
            .map(StoreMigrationInventoryEntryCount::get),
        Ok(StoreMigrationInventoryEntryCount::MAXIMUM)
    );
    assert_eq!(
        StoreMigrationInventoryEntryCount::new(2_097_153),
        Err(StoreMigrationInventoryEntryCountError::AboveMaximum {
            observed: 2_097_153,
            maximum: 2_097_152,
        })
    );
}

fn frozen_entries()
-> Result<(StoreMigrationInventoryEntry, StoreMigrationInventoryEntry), Box<dyn Error>> {
    let segment_bytes = fixture(SEGMENT)?;
    let catalog_bytes = fixture(CATALOG)?;
    let segment = AdmittedSegment::decode(&segment_bytes, maximum_policy())?;
    let segment_entry = StoreMigrationInventoryEntry::from_segment(&segment);
    let catalog = ChecksummedCatalog::decode(&catalog_bytes)?.admit(&[segment])?;
    let catalog_entry = StoreMigrationInventoryEntry::from_catalog(&catalog);
    Ok((segment_entry, catalog_entry))
}

const fn maximum_policy() -> SegmentReadPolicy {
    SegmentReadPolicy::new(SegmentRecordLimit::MAXIMUM, LayoutEntryLimit::MAXIMUM)
}

fn fixture(hex: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    decode_hex(hex.strip_suffix('\n').ok_or("fixture must end in one LF")?).map_err(Into::into)
}
