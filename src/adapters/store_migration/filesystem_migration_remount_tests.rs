//! This module owns restart admission laws for changed root coordinates.

use std::error::Error;
use std::fs;
use std::path::Path;

use super::filesystem_migration_test_fixture::open_authority;
use super::migration_catalog_coordinates::MigrationCatalogCoordinates;
use super::store_root_identity::StoreRootIdentities;
use super::{
    AdmittedStoreMigrationIntent, CanonicalStoreFormatMarker, CanonicalStoreMigrationIntent,
    CanonicalStoreMigrationReceipt, StoreRootDeviceIdentity, StoreRootFileIdentity,
    StoreRootIdentityCoordinate, StoreRootMountIdentity, execute_store_migration,
};
use crate::adapters::filesystem_test_sandbox::TestDirectory;
use crate::adapters::{FilesystemPlatformAdmissionError, FilesystemVersionTwoAdmission};

#[test]
fn a_remounted_store_reopens_without_changing_its_catalog_head() -> Result<(), Box<dyn Error>> {
    let sandbox = migrated("migration-remount-admission")?;
    let head = fs::read(sandbox.path().join("HEAD"))?;
    let _coordinates = substitute_coordinate(&sandbox, StoreRootIdentityCoordinate::Mount)?;
    let intent = fs::read(sandbox.path().join("migration.intent"))?;
    let receipt = fs::read(sandbox.path().join("migration.receipt"))?;

    let admission = reopen(sandbox.path())?;

    assert_eq!(fs::read(sandbox.path().join("HEAD"))?, head);
    assert_eq!(fs::read(sandbox.path().join("migration.intent"))?, intent);
    assert_eq!(fs::read(sandbox.path().join("migration.receipt"))?, receipt);
    drop(admission);
    Ok(())
}

#[test]
fn device_and_inode_disagreement_refuse_with_exact_coordinates() -> Result<(), Box<dyn Error>> {
    for coordinate in [
        StoreRootIdentityCoordinate::Device,
        StoreRootIdentityCoordinate::File,
    ] {
        let sandbox = migrated(&format!("migration-moved-root-{coordinate:?}"))?;
        let head = fs::read(sandbox.path().join("HEAD"))?;
        let (expected, observed) = substitute_coordinate(&sandbox, coordinate)?;

        let error = reopen(sandbox.path())
            .err()
            .ok_or("a moved root was admitted")?;

        assert!(matches!(
            error,
            FilesystemPlatformAdmissionError::RootIdentityChanged {
                coordinate: refused, expected: bound, observed: current,
            } if refused == coordinate && bound == expected && current == observed
        ));
        assert_eq!(fs::read(sandbox.path().join("HEAD"))?, head);
    }
    Ok(())
}

fn migrated(name: &str) -> Result<TestDirectory, Box<dyn Error>> {
    let (sandbox, mut authority) = open_authority(name)?;
    let intent = authority.observe_intent()?;
    let _receipt = execute_store_migration(&mut authority, &intent)?;
    drop(authority);
    Ok(sandbox)
}

fn substitute_coordinate(
    sandbox: &TestDirectory,
    coordinate: StoreRootIdentityCoordinate,
) -> Result<(u64, u64), Box<dyn Error>> {
    let bytes = fs::read(sandbox.path().join("migration.intent"))?;
    let original = AdmittedStoreMigrationIntent::decode(&bytes)?;
    let observed = match coordinate {
        StoreRootIdentityCoordinate::Device => original.root_device_identity().get(),
        StoreRootIdentityCoordinate::Mount => original.root_mount_identity().get(),
        StoreRootIdentityCoordinate::File => original.root_file_identity().get(),
    };
    let expected = observed
        .checked_add(1)
        .ok_or("fixture coordinate overflow")?;
    let changed = |selected, value| {
        if coordinate == selected {
            expected
        } else {
            value
        }
    };
    let roots = StoreRootIdentities::new(
        StoreRootDeviceIdentity::from_admitted(changed(
            StoreRootIdentityCoordinate::Device,
            original.root_device_identity().get(),
        )),
        StoreRootMountIdentity::from_admitted(changed(
            StoreRootIdentityCoordinate::Mount,
            original.root_mount_identity().get(),
        )),
        StoreRootFileIdentity::from_admitted(changed(
            StoreRootIdentityCoordinate::File,
            original.root_file_identity().get(),
        )),
    );
    let catalog = MigrationCatalogCoordinates::new(
        original.catalog_generation(),
        original.catalog_length(),
        original.catalog_digest(),
        original.predecessor_catalog_digest(),
    );
    let intent = CanonicalStoreMigrationIntent::from_coordinates(
        catalog,
        original.inventory_digest(),
        roots,
    );
    let receipt = CanonicalStoreMigrationReceipt::from_canonical(
        &intent,
        &CanonicalStoreFormatMarker::version_two(),
    );
    fs::write(sandbox.path().join("migration.intent"), intent.encoded())?;
    fs::write(sandbox.path().join("migration.receipt"), receipt.encoded())?;
    Ok((expected, observed))
}

#[cfg(target_os = "linux")]
fn reopen(root: &Path) -> Result<FilesystemVersionTwoAdmission, FilesystemPlatformAdmissionError> {
    FilesystemVersionTwoAdmission::reopen(root)
}

#[cfg(not(target_os = "linux"))]
fn reopen(root: &Path) -> Result<FilesystemVersionTwoAdmission, FilesystemPlatformAdmissionError> {
    FilesystemVersionTwoAdmission::reopen_unchecked_for_repository_tasks(root)
}
