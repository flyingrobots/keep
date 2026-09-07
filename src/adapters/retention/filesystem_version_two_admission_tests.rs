//! Version-two reopen laws: writer authority requires jointly admitted migration records.

use std::error::Error;
use std::fs;

use super::filesystem_retention_test_fixture::migrated_store;
use crate::adapters::{FilesystemPlatformAdmission, FilesystemPlatformAdmissionError};

#[test]
fn version_two_reopen_refuses_a_corrupt_format_marker() -> Result<(), Box<dyn Error>> {
    let sandbox = migrated_store("version-two-admission-corrupt-marker")?;
    let marker = sandbox.path().join("FORMAT");
    let mut bytes = fs::read(&marker)?;
    *bytes
        .get_mut(40)
        .ok_or("format marker shorter than 41 bytes")? ^= 0x01;
    fs::write(&marker, &bytes)?;

    let error = FilesystemPlatformAdmission::reopen_version_two_unchecked_for_tests(sandbox.path())
        .err()
        .ok_or("corrupt format marker was unexpectedly admitted")?;

    assert!(matches!(
        error,
        FilesystemPlatformAdmissionError::MigrationRecord { .. }
    ));
    sandbox.remove()?;
    Ok(())
}

#[test]
fn version_two_reopen_refuses_an_oversized_format_marker() -> Result<(), Box<dyn Error>> {
    let sandbox = migrated_store("version-two-admission-oversized-marker")?;
    let marker = sandbox.path().join("FORMAT");
    let mut bytes = fs::read(&marker)?;
    bytes.push(0);
    fs::write(&marker, &bytes)?;

    let error = FilesystemPlatformAdmission::reopen_version_two_unchecked_for_tests(sandbox.path())
        .err()
        .ok_or("oversized format marker was unexpectedly admitted")?;

    assert!(matches!(
        error,
        FilesystemPlatformAdmissionError::MigrationRecord { .. }
    ));
    sandbox.remove()?;
    Ok(())
}

#[test]
fn version_two_reopen_refuses_a_receipt_that_disagrees_with_its_intent()
-> Result<(), Box<dyn Error>> {
    let sandbox = migrated_store("version-two-admission-receipt-disagrees")?;
    let intent = fs::read(sandbox.path().join("migration.intent"))?;
    fs::write(sandbox.path().join("migration.receipt"), &intent)?;

    let error = FilesystemPlatformAdmission::reopen_version_two_unchecked_for_tests(sandbox.path())
        .err()
        .ok_or("receipt disagreeing with its intent was unexpectedly admitted")?;

    assert!(matches!(
        error,
        FilesystemPlatformAdmissionError::MigrationRecord { .. }
    ));
    sandbox.remove()?;
    Ok(())
}

#[test]
fn version_two_reopen_admits_exact_migration_records() -> Result<(), Box<dyn Error>> {
    let sandbox = migrated_store("version-two-admission-exact")?;

    let admission =
        FilesystemPlatformAdmission::reopen_version_two_unchecked_for_tests(sandbox.path())?;

    drop(admission);
    sandbox.remove()?;
    Ok(())
}
