//! Filesystem migration segment-pool refusal laws.

use std::error::Error;
use std::fs;
use std::io;

use super::filesystem_inventory_error::{
    FilesystemMigrationInventoryError, MigrationInventoryPool,
};
use super::filesystem_inventory_segments;
use super::filesystem_inventory_segments_test_fixture::{
    SegmentPoolFixture, maximum_policy, one_zero_bytes,
};
use crate::adapters::{
    AdmittedSegment, CatalogRestartError, CatalogRestartPhase, RecoveryPoolNameError,
    physical_pool_name,
};

#[test]
fn noncanonical_segment_name_refuses_inventory() -> Result<(), Box<dyn Error>> {
    let fixture = SegmentPoolFixture::create("migration-segment-name-refusal")?;
    let bytes = one_zero_bytes()?;
    let segment = AdmittedSegment::decode(&bytes, maximum_policy())?;
    let canonical = physical_pool_name::segment(segment.digest());
    let stem = canonical
        .strip_suffix(".seg")
        .ok_or_else(|| io::Error::other("canonical segment name lost its suffix"))?;
    fixture.write_named(&format!("{}.seg", stem.to_uppercase()), &bytes)?;

    let pool = fixture.open()?;
    let error = require_error(filesystem_inventory_segments::read(
        &pool,
        1,
        maximum_policy(),
    ))?;
    assert!(matches!(
        error,
        FilesystemMigrationInventoryError::Name {
            pool: MigrationInventoryPool::Segments,
            source: RecoveryPoolNameError::UppercaseDigest,
            ..
        }
    ));
    drop(pool);
    fixture.remove()?;
    Ok(())
}

#[test]
fn corrupt_segment_bytes_refuse_inventory() -> Result<(), Box<dyn Error>> {
    let fixture = SegmentPoolFixture::create("migration-segment-corruption-refusal")?;
    let mut bytes = one_zero_bytes()?;
    let segment = AdmittedSegment::decode(&bytes, maximum_policy())?;
    let name = physical_pool_name::segment(segment.digest());
    let first = bytes
        .first_mut()
        .ok_or_else(|| io::Error::other("segment fixture is empty"))?;
    *first ^= u8::MAX;
    fixture.write_named(&name, &bytes)?;

    let pool = fixture.open()?;
    let error = require_error(filesystem_inventory_segments::read(
        &pool,
        1,
        maximum_policy(),
    ))?;
    let FilesystemMigrationInventoryError::Artifact {
        pool: observed_pool,
        source,
        ..
    } = error
    else {
        return Err(io::Error::other("corrupt segment returned wrong refusal").into());
    };
    assert_eq!(observed_pool, MigrationInventoryPool::Segments);
    assert!(matches!(
        source.as_ref(),
        CatalogRestartError::Segment { .. }
    ));
    drop(pool);
    fixture.remove()?;
    Ok(())
}

#[cfg(unix)]
#[test]
fn linked_segment_entry_refuses_inventory() -> Result<(), Box<dyn Error>> {
    use std::os::unix::fs::symlink;

    let fixture = SegmentPoolFixture::create("migration-segment-link-refusal")?;
    let bytes = one_zero_bytes()?;
    let segment = AdmittedSegment::decode(&bytes, maximum_policy())?;
    let name = physical_pool_name::segment(segment.digest());
    fs::write(fixture.path().join("linked-target"), &bytes)?;
    symlink("../linked-target", fixture.pool_path().join(name))?;

    let pool = fixture.open()?;
    let error = require_error(filesystem_inventory_segments::read(
        &pool,
        1,
        maximum_policy(),
    ))?;
    let FilesystemMigrationInventoryError::Artifact {
        pool: observed_pool,
        source,
        ..
    } = error
    else {
        return Err(io::Error::other("linked segment returned wrong refusal").into());
    };
    assert_eq!(observed_pool, MigrationInventoryPool::Segments);
    assert!(matches!(
        source.as_ref(),
        CatalogRestartError::Io {
            phase: CatalogRestartPhase::OpenSegment,
            ..
        }
    ));
    drop(pool);
    fixture.remove()?;
    Ok(())
}

#[test]
fn segment_pool_above_remaining_limit_refuses_before_names() -> Result<(), Box<dyn Error>> {
    let fixture = SegmentPoolFixture::create("migration-segment-limit-refusal")?;
    fixture.write_named("unknown", b"not admitted")?;

    let pool = fixture.open()?;
    let error = require_error(filesystem_inventory_segments::read(
        &pool,
        0,
        maximum_policy(),
    ))?;
    assert!(matches!(
        error,
        FilesystemMigrationInventoryError::EntryLimitExceeded {
            pool: MigrationInventoryPool::Segments,
            maximum: 0,
            observed_at_least: 1,
        }
    ));
    drop(pool);
    fixture.remove()?;
    Ok(())
}

fn require_error<T>(
    result: Result<T, FilesystemMigrationInventoryError>,
) -> Result<FilesystemMigrationInventoryError, io::Error> {
    result.map_or_else(Ok, |_value| {
        Err(io::Error::other(
            "filesystem migration segment inventory unexpectedly succeeded",
        ))
    })
}
