//! Filesystem migration catalog-pool refusal and orphan laws.

use std::error::Error;
use std::fs;
use std::io;

use super::filesystem_inventory_catalogs;
use super::filesystem_inventory_catalogs_test_fixture::{
    CatalogPoolFixture, empty_segment_bytes, maximum_policy,
};
use super::filesystem_inventory_error::FilesystemMigrationInventoryError;
use super::filesystem_inventory_segments;
use crate::adapters::{
    AdmittedSegment, CatalogAdmissionError, CatalogRestartError, physical_pool_name,
};

#[test]
fn unrelated_orphan_segment_remains_in_exact_pool_inventory() -> Result<(), Box<dyn Error>> {
    let fixture = CatalogPoolFixture::create("migration-catalog-orphan-segment")?;
    let orphan_bytes = empty_segment_bytes()?;
    let orphan = AdmittedSegment::decode(&orphan_bytes, maximum_policy())?;
    fs::write(
        fixture
            .segments_path()
            .join(physical_pool_name::segment(orphan.digest())),
        &orphan_bytes,
    )?;
    let segments_directory = fixture.open_segments()?;
    let catalogs_directory = fixture.open_catalogs()?;
    let segments = filesystem_inventory_segments::read(&segments_directory, 2, maximum_policy())?;

    let catalogs = filesystem_inventory_catalogs::read(
        &catalogs_directory,
        &segments_directory,
        &segments,
        1,
        maximum_policy(),
    )?;
    assert_eq!(catalogs.entries().len(), 1);
    drop(catalogs_directory);
    drop(segments_directory);
    fixture.remove()?;
    Ok(())
}

#[test]
fn catalog_missing_its_segment_refuses_inventory() -> Result<(), Box<dyn Error>> {
    let fixture = CatalogPoolFixture::create("migration-catalog-missing-segment")?;
    let missing = AdmittedSegment::decode(fixture.segment_bytes(), maximum_policy())?.digest();
    remove_fixture_segment(&fixture)?;
    let segments_directory = fixture.open_segments()?;
    let catalogs_directory = fixture.open_catalogs()?;
    let segments = filesystem_inventory_segments::read(&segments_directory, 0, maximum_policy())?;

    let error = require_error(filesystem_inventory_catalogs::read(
        &catalogs_directory,
        &segments_directory,
        &segments,
        1,
        maximum_policy(),
    ))?;
    let FilesystemMigrationInventoryError::ReferencedSegment { digest, source } = error else {
        return Err(io::Error::other("missing segment returned wrong refusal").into());
    };
    assert_eq!(digest, missing);
    assert!(matches!(
        source.as_ref(),
        CatalogRestartError::CatalogAdmission {
            source
        } if matches!(
            source.as_ref(),
            CatalogAdmissionError::MissingSegment { digest } if *digest == missing
        )
    ));
    drop(catalogs_directory);
    drop(segments_directory);
    fixture.remove()?;
    Ok(())
}

#[test]
fn segment_corruption_after_pool_admission_refuses_catalog() -> Result<(), Box<dyn Error>> {
    let fixture = CatalogPoolFixture::create("migration-catalog-segment-corruption")?;
    let segments_directory = fixture.open_segments()?;
    let catalogs_directory = fixture.open_catalogs()?;
    let segments = filesystem_inventory_segments::read(&segments_directory, 1, maximum_policy())?;
    let segment = AdmittedSegment::decode(fixture.segment_bytes(), maximum_policy())?;
    fs::write(
        fixture
            .segments_path()
            .join(physical_pool_name::segment(segment.digest())),
        b"corrupt",
    )?;

    let error = require_error(filesystem_inventory_catalogs::read(
        &catalogs_directory,
        &segments_directory,
        &segments,
        1,
        maximum_policy(),
    ))?;
    let (digest, source) = match error {
        FilesystemMigrationInventoryError::ReferencedSegment { digest, source } => (digest, source),
        other => {
            return Err(io::Error::other(format!(
                "corrupt referenced segment returned wrong refusal: {other:?}"
            ))
            .into());
        }
    };
    assert_eq!(digest, segment.digest());
    assert!(matches!(
        source.as_ref(),
        CatalogRestartError::Segment {
            expected,
            ..
        } if *expected == segment.digest()
    ));
    drop(catalogs_directory);
    drop(segments_directory);
    fixture.remove()?;
    Ok(())
}

#[test]
fn valid_segment_substitution_names_the_referenced_coordinate() -> Result<(), Box<dyn Error>> {
    let fixture = CatalogPoolFixture::create("migration-catalog-segment-substitution")?;
    let segments_directory = fixture.open_segments()?;
    let catalogs_directory = fixture.open_catalogs()?;
    let segments = filesystem_inventory_segments::read(&segments_directory, 1, maximum_policy())?;
    let expected = AdmittedSegment::decode(fixture.segment_bytes(), maximum_policy())?;
    let replacement_bytes = empty_segment_bytes()?;
    let replacement = AdmittedSegment::decode(&replacement_bytes, maximum_policy())?;
    assert_ne!(expected.digest(), replacement.digest());
    fs::write(
        fixture
            .segments_path()
            .join(physical_pool_name::segment(expected.digest())),
        &replacement_bytes,
    )?;

    let error = require_error(filesystem_inventory_catalogs::read(
        &catalogs_directory,
        &segments_directory,
        &segments,
        1,
        maximum_policy(),
    ))?;
    let FilesystemMigrationInventoryError::ReferencedSegment { digest, source } = error else {
        return Err(io::Error::other("segment substitution returned wrong refusal").into());
    };
    assert_eq!(digest, expected.digest());
    assert!(matches!(
        source.as_ref(),
        CatalogRestartError::SegmentCoordinate {
            expected: expected_digest,
            observed
        } if *expected_digest == expected.digest() && *observed == replacement.digest()
    ));
    drop(catalogs_directory);
    drop(segments_directory);
    fixture.remove()?;
    Ok(())
}

fn remove_fixture_segment(fixture: &CatalogPoolFixture) -> Result<(), Box<dyn Error>> {
    let segment = AdmittedSegment::decode(fixture.segment_bytes(), maximum_policy())?;
    fs::remove_file(
        fixture
            .segments_path()
            .join(physical_pool_name::segment(segment.digest())),
    )?;
    Ok(())
}

fn require_error<T>(
    result: Result<T, FilesystemMigrationInventoryError>,
) -> Result<FilesystemMigrationInventoryError, io::Error> {
    result.map_or_else(Ok, |_value| {
        Err(io::Error::other(
            "filesystem migration catalog inventory unexpectedly succeeded",
        ))
    })
}
