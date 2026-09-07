//! Version-one recovery adapters refuse a migrated version-two root.

use std::error::Error;

use super::filesystem_retention_test_fixture::migrated_store;
use crate::adapters::{
    FilesystemRecoveryInventoryReader, FilesystemRecoveryStageDiscardOpenError,
    FilesystemRecoveryStageDiscarder, RecoveryInventoryError, RecoveryInventoryOperation,
    RecoveryNamespace,
};

#[test]
fn recovery_inventory_reader_refuses_a_migrated_root() -> Result<(), Box<dyn Error>> {
    let sandbox = migrated_store("recovery-admission-inventory-migrated")?;

    let error = FilesystemRecoveryInventoryReader::open_unchecked_for_tests(sandbox.path())
        .err()
        .ok_or("version-one recovery inventory opened a migrated root")?;

    assert!(matches!(
        error,
        RecoveryInventoryError::Io {
            namespace: RecoveryNamespace::Root,
            operation: RecoveryInventoryOperation::OpenNamespace,
            ..
        }
    ));
    sandbox.remove()?;
    Ok(())
}

#[test]
fn recovery_stage_discarder_refuses_a_migrated_root() -> Result<(), Box<dyn Error>> {
    let sandbox = migrated_store("recovery-admission-discarder-migrated")?;

    let error = FilesystemRecoveryStageDiscarder::open_unchecked_for_tests(sandbox.path())
        .err()
        .ok_or("version-one recovery discarder opened a migrated root")?;

    assert!(matches!(
        error,
        FilesystemRecoveryStageDiscardOpenError::Namespace { .. }
    ));
    sandbox.remove()?;
    Ok(())
}
