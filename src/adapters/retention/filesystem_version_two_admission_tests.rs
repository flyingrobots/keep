//! Version-two reopen laws: writer authority requires jointly admitted migration records.

use std::error::Error;
use std::fs;

use super::filesystem_retention_test_fixture::migrated_store;
use crate::adapters::filesystem_root_identity::FilesystemRootIdentity;
use crate::adapters::filesystem_version_two_admission::{BoundRootIdentity, require_root_identity};
use crate::adapters::{
    FilesystemPlatformAdmissionError, FilesystemVersionTwoAdmission, StoreRootIdentityCoordinate,
    VersionTwoRecordRefusal,
};

#[test]
fn version_two_reopen_refuses_a_corrupt_format_marker() -> Result<(), Box<dyn Error>> {
    let sandbox = migrated_store("version-two-admission-corrupt-marker")?;
    let marker = sandbox.path().join("FORMAT");
    let mut bytes = fs::read(&marker)?;
    *bytes
        .get_mut(40)
        .ok_or("format marker shorter than 41 bytes")? ^= 0x01;
    fs::write(&marker, &bytes)?;

    let error = FilesystemVersionTwoAdmission::reopen_unchecked_for_tests(sandbox.path())
        .err()
        .ok_or("corrupt format marker was unexpectedly admitted")?;

    let FilesystemPlatformAdmissionError::MigrationRecord { source } = error else {
        return Err("corrupt format marker refused outside record admission".into());
    };
    assert!(matches!(
        source
            .get_ref()
            .and_then(|refusal| refusal.downcast_ref::<VersionTwoRecordRefusal>()),
        Some(VersionTwoRecordRefusal::Marker { .. })
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

    let error = FilesystemVersionTwoAdmission::reopen_unchecked_for_tests(sandbox.path())
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

    let error = FilesystemVersionTwoAdmission::reopen_unchecked_for_tests(sandbox.path())
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

    let admission = FilesystemVersionTwoAdmission::reopen_unchecked_for_tests(sandbox.path())?;

    drop(admission);
    sandbox.remove()?;
    Ok(())
}

#[cfg(target_os = "linux")]
#[test]
fn production_version_two_reopen_refuses_an_aliased_protocol_directory()
-> Result<(), Box<dyn Error>> {
    let sandbox = migrated_store("version-two-admission-aliased-gc")?;
    let alias_target = sandbox.path().join("elsewhere");
    fs::create_dir(&alias_target)?;
    fs::remove_dir(sandbox.path().join("gc"))?;
    std::os::unix::fs::symlink(&alias_target, sandbox.path().join("gc"))?;

    let error = FilesystemVersionTwoAdmission::reopen(sandbox.path())
        .err()
        .ok_or("aliased gc protocol directory was unexpectedly admitted")?;

    assert!(matches!(
        error,
        FilesystemPlatformAdmissionError::Platform { .. }
    ));
    sandbox.remove()?;
    Ok(())
}

#[cfg(target_os = "linux")]
#[test]
fn production_version_two_reopen_admits_an_exact_migrated_store() -> Result<(), Box<dyn Error>> {
    let sandbox = migrated_store("version-two-admission-production-exact")?;

    let admission = FilesystemVersionTwoAdmission::reopen(sandbox.path())?;

    drop(admission);
    sandbox.remove()?;
    Ok(())
}

#[test]
fn reopened_root_identity_must_match_the_intent_coordinates() {
    let bound = BoundRootIdentity::new(1, 2, 3);

    assert!(require_root_identity(bound, FilesystemRootIdentity::new(1, 2, 3)).is_ok());
    assert!(matches!(
        require_root_identity(bound, FilesystemRootIdentity::new(1, 2, 4)),
        Err(FilesystemPlatformAdmissionError::RootIdentityChanged {
            coordinate: StoreRootIdentityCoordinate::File,
            expected: 3,
            observed: 4,
        })
    ));
    assert!(matches!(
        require_root_identity(bound, FilesystemRootIdentity::new(9, 2, 3)),
        Err(FilesystemPlatformAdmissionError::RootIdentityChanged {
            coordinate: StoreRootIdentityCoordinate::Device,
            ..
        })
    ));
    assert!(matches!(
        require_root_identity(bound, FilesystemRootIdentity::new(1, 7, 3)),
        Err(FilesystemPlatformAdmissionError::RootIdentityChanged {
            coordinate: StoreRootIdentityCoordinate::Mount,
            ..
        })
    ));
}

fn refuses_namespace(
    name: &str,
    mutate: impl FnOnce(&std::path::Path) -> Result<(), Box<dyn Error>>,
) -> Result<(), Box<dyn Error>> {
    let sandbox = migrated_store(name)?;
    mutate(sandbox.path())?;

    let error = FilesystemVersionTwoAdmission::reopen_unchecked_for_tests(sandbox.path())
        .err()
        .ok_or_else(|| format!("{name}: version-two root was unexpectedly admitted"))?;

    assert!(
        matches!(error, FilesystemPlatformAdmissionError::Namespace { .. }),
        "{name}: expected a Namespace refusal, got {error:?}"
    );
    sandbox.remove()?;
    Ok(())
}

#[test]
fn version_two_reopen_refuses_a_missing_dispositions_directory() -> Result<(), Box<dyn Error>> {
    refuses_namespace("version-two-admission-no-dispositions", |root| {
        fs::remove_dir(root.join("recovery").join("dispositions"))?;
        Ok(())
    })
}

#[test]
fn version_two_reopen_refuses_a_stray_gc_entry() -> Result<(), Box<dyn Error>> {
    refuses_namespace("version-two-admission-gc-junk", |root| {
        fs::write(root.join("gc").join("junk"), b"")?;
        Ok(())
    })
}

#[test]
fn version_two_reopen_refuses_a_stray_recovery_entry() -> Result<(), Box<dyn Error>> {
    refuses_namespace("version-two-admission-recovery-junk", |root| {
        fs::write(root.join("recovery").join("junk"), b"")?;
        Ok(())
    })
}

#[test]
fn version_two_reopen_refuses_a_missing_retention_pool() -> Result<(), Box<dyn Error>> {
    refuses_namespace("version-two-admission-no-roots", |root| {
        fs::remove_dir(root.join("retention").join("roots"))?;
        Ok(())
    })
}
