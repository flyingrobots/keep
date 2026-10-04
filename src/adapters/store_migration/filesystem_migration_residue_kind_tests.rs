//! This module owns typed refusal of non-regular migration residue.

#![cfg(unix)]

use std::error::Error;
use std::fs;
use std::os::unix::fs::symlink;

use super::StoreMigrationRecoveryStorage;
use super::filesystem_migration_test_fixture::open_authority;
use crate::adapters::filesystem_exact_record::{ExactRecordError, ExactRecordRefusal};

#[test]
fn symlink_residue_refuses_by_kind_before_reading_its_target() -> Result<(), Box<dyn Error>> {
    for name in [
        "migration.intent.next",
        "migration.intent",
        "FORMAT.next",
        "FORMAT",
        "migration.receipt.next",
        "migration.receipt",
    ] {
        let (sandbox, mut authority) = open_authority("migration-residue-symlink")?;
        let target = sandbox.path().join("HEAD");
        let before = fs::read(&target)?;
        symlink("HEAD", sandbox.path().join(name))?;
        let error = StoreMigrationRecoveryStorage::observe_residue(&mut authority)
            .err()
            .ok_or("symlink residue was admitted")?;
        assert!(
            matches!(
                error
                    .get_ref()
                    .and_then(|source| source.downcast_ref::<ExactRecordError>()),
                Some(ExactRecordError::Refused(ExactRecordRefusal::KindOrLength))
            ),
            "{name}: {error:?}"
        );
        assert_eq!(fs::read(&target)?, before);
        assert_eq!(
            fs::read_link(sandbox.path().join(name))?,
            std::path::Path::new("HEAD")
        );
        drop(authority);
        sandbox.remove()?;
    }
    Ok(())
}
