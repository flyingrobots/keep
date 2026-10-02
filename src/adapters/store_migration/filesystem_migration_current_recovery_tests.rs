//! This module owns recovery's current-authority validation before admission.

use std::error::Error;
use std::fs;
use std::io;

use super::filesystem_migration_test_fixture::{maximum_policy, open_authority};
use super::{
    FilesystemMigrationAuthorityError, FilesystemStoreMigrationAuthority, recover_store_migration,
};

#[test]
fn a_corrupt_current_head_cannot_receive_a_version_one_recovery_receipt()
-> Result<(), Box<dyn Error>> {
    let (sandbox, authority) = open_authority("migration-recovery-current-head")?;
    let expected = authority.observe_intent()?;
    drop(authority);
    let path = sandbox.path().join("HEAD");
    let mut bytes = fs::read(&path)?;
    *bytes.last_mut().ok_or("HEAD is empty")? ^= 1;
    fs::write(&path, &bytes)?;
    let mut recovered = FilesystemStoreMigrationAuthority::reopen_for_recovery_unchecked_for_tests(
        sandbox.path(),
        maximum_policy(),
    )?;
    let error = recover_store_migration(&mut recovered, &expected)
        .err()
        .ok_or("corrupt HEAD received a version-one recovery receipt")?;
    assert!(matches!(
        error,
        super::StoreMigrationRecoveryError::CurrentVerification { .. }
    ));
    let source = error
        .source()
        .and_then(|source| source.downcast_ref::<io::Error>())
        .ok_or("current-state failure lost its I/O boundary")?;
    assert!(matches!(
        source
            .get_ref()
            .and_then(|source| source.downcast_ref::<FilesystemMigrationAuthorityError>()),
        Some(FilesystemMigrationAuthorityError::Head {
            source: crate::adapters::PublicationHeadDecodeError::ChecksumMismatch { .. }
        })
    ));
    assert_eq!(fs::read(&path)?, bytes);
    assert!(!sandbox.path().join("migration.intent.next").exists());
    drop(recovered);
    sandbox.remove()?;
    Ok(())
}
