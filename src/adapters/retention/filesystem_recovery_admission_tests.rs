//! Version-one recovery adapters refuse a version-two root.

use std::error::Error;

use super::filesystem_retention_test_fixture::initialized_store;
use crate::adapters::{
    FilesystemRecoveryInventoryReader, FilesystemRecoveryStageDiscardOpenError,
    FilesystemRecoveryStageDiscarder, RecoveryInventoryError, RecoveryInventoryOperation,
    RecoveryNamespace,
};

#[test]
fn recovery_inventory_reader_refuses_a_version_two_root() -> Result<(), Box<dyn Error>> {
    let sandbox = initialized_store("recovery-admission-inventory-version-two")?;

    let error = FilesystemRecoveryInventoryReader::open_unchecked_for_tests(sandbox.path())
        .err()
        .ok_or("version-one recovery inventory opened a version-two root")?;

    assert!(matches!(
        error,
        RecoveryInventoryError::Io {
            namespace: RecoveryNamespace::Root,
            operation: RecoveryInventoryOperation::OpenNamespace,
            ..
        }
    ));
    Ok(())
}

#[test]
fn recovery_stage_discarder_refuses_a_version_two_root() -> Result<(), Box<dyn Error>> {
    let sandbox = initialized_store("recovery-admission-discarder-version-two")?;

    let error = FilesystemRecoveryStageDiscarder::open_unchecked_for_tests(sandbox.path())
        .err()
        .ok_or("version-one recovery discarder opened a version-two root")?;

    assert!(matches!(
        error,
        FilesystemRecoveryStageDiscardOpenError::Namespace { .. }
    ));
    Ok(())
}
