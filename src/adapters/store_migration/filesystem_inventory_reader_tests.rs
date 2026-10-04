//! Writer-locked filesystem migration inventory laws.

use std::error::Error;
use std::fs;

use super::filesystem_inventory_reader::FilesystemStoreMigrationInventoryReader;
use super::{FilesystemMigrationInventoryError, MigrationInventoryPool};
use crate::adapters::filesystem_test_sandbox::TestDirectory;
use crate::adapters::test_support::decode_hex;
use crate::adapters::{
    AdmittedSegment, ChecksummedCatalog, FilesystemPlatformAdmission, physical_pool_name,
};

const SEGMENT_HEX: &str =
    include_str!("../../../conformance/segment-store/v1/one-zero-segment.hex");
const CATALOG_HEX: &str =
    include_str!("../../../conformance/segment-store/v1/one-zero-catalog.hex");
const INVENTORY_DIGEST: &str = "40bf5d49c34847ac9cf46a256f343cee80cd980d1405d2dd02ceff8f58d674f9";

#[test]
fn writer_locked_pools_reproduce_the_frozen_inventory_digest() -> Result<(), Box<dyn Error>> {
    let sandbox = TestDirectory::create("migration-inventory-reader")?;
    let admission = FilesystemPlatformAdmission::initialize_unchecked_for_tests(sandbox.path())?;
    let segment_bytes = decode_hex(SEGMENT_HEX.trim())?;
    let catalog_bytes = decode_hex(CATALOG_HEX.trim())?;
    let policy = super::filesystem_inventory_catalogs_test_fixture::maximum_policy();
    let segment = AdmittedSegment::decode(&segment_bytes, policy)?;
    let catalog = ChecksummedCatalog::decode(&catalog_bytes)?;
    fs::write(
        sandbox
            .path()
            .join("segments")
            .join(physical_pool_name::segment(segment.digest())),
        &segment_bytes,
    )?;
    fs::write(
        sandbox
            .path()
            .join("catalogs")
            .join(physical_pool_name::catalog(
                catalog.generation(),
                catalog.digest(),
            )),
        &catalog_bytes,
    )?;

    let reader = FilesystemStoreMigrationInventoryReader::open(admission, policy)?;
    let digest = reader.read()?;
    assert_eq!(digest.as_bytes().as_slice(), decode_hex(INVENTORY_DIGEST)?);
    drop(reader);
    sandbox.remove()?;
    Ok(())
}

#[test]
fn replaced_pool_directory_refuses_before_artifact_reads() -> Result<(), Box<dyn Error>> {
    let sandbox = TestDirectory::create("migration-inventory-directory-replacement")?;
    let admission = FilesystemPlatformAdmission::initialize_unchecked_for_tests(sandbox.path())?;
    let policy = super::filesystem_inventory_catalogs_test_fixture::maximum_policy();
    let reader = FilesystemStoreMigrationInventoryReader::open(admission, policy)?;
    fs::rename(
        sandbox.path().join("segments"),
        sandbox.path().join("segments-replaced"),
    )?;
    fs::create_dir(sandbox.path().join("segments"))?;

    let error = reader
        .read()
        .err()
        .ok_or("replaced segment pool unexpectedly passed")?;
    assert!(matches!(
        error,
        FilesystemMigrationInventoryError::NamespaceChanged {
            pool: MigrationInventoryPool::Segments
        }
    ));
    drop(reader);
    sandbox.remove()?;
    Ok(())
}
