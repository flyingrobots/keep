//! Filesystem migration catalog-pool admission laws.

use std::error::Error;

use super::StoreMigrationInventoryEntry;
use super::filesystem_inventory_catalogs;
use super::filesystem_inventory_catalogs_test_fixture::{CatalogPoolFixture, maximum_policy};
use super::filesystem_inventory_segments;
use crate::adapters::{AdmittedSegment, ChecksummedCatalog};

#[test]
fn every_catalog_binds_exact_pool_segment_records() -> Result<(), Box<dyn Error>> {
    let fixture = CatalogPoolFixture::create("migration-catalog-inventory")?;
    let segments_directory = fixture.open_segments()?;
    let catalogs_directory = fixture.open_catalogs()?;
    let segments = filesystem_inventory_segments::read(&segments_directory, 1, maximum_policy())?;
    let catalogs = filesystem_inventory_catalogs::read(
        &catalogs_directory,
        &segments_directory,
        &segments,
        1,
        maximum_policy(),
    )?;
    let segment = AdmittedSegment::decode(fixture.segment_bytes(), maximum_policy())?;
    let admitted = ChecksummedCatalog::decode(fixture.catalog_bytes())?.admit(&[segment])?;

    assert_eq!(
        catalogs.entries(),
        &[StoreMigrationInventoryEntry::from_catalog(&admitted)]
    );
    drop(catalogs_directory);
    drop(segments_directory);
    fixture.remove()?;
    Ok(())
}
