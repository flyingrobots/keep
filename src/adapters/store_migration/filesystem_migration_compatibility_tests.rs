//! Migration preserves version-one bytes while irrevocably refusing version-one authority.

use super::filesystem_migration_recovery_tests::version_one_witness;
use super::filesystem_migration_test_fixture::{maximum_policy, open_authority};
use super::migration_resumption::{MigrationRecords, execute_phase};
use super::{StoreMigrationPhase, StoreMigrationStorage};
use crate::adapters::{FilesystemPlatformAdmission, FilesystemPlatformAdmissionError};
use std::{error::Error, io};

// Size: medium. Oracle: KEEP-MIGRATION-008 forbids v1 authority after the first
// migration effect, while every original segment/catalog/head byte is preserved.
// Delete only if the one-way migration contract disappears or stronger runtime laws subsume it.
#[test]
fn every_migration_prefix_preserves_v1_bytes_but_refuses_v1_authority() -> Result<(), Box<dyn Error>>
{
    for count in 0..=StoreMigrationPhase::ALL.len() {
        let (store, mut authority) =
            open_authority(&format!("migration-v1-compatibility-{count}"))?;
        let before = version_one_witness(store.path())?;
        let intent = authority.observe_intent()?;
        StoreMigrationStorage::verify_current(&mut authority, &intent)?;
        let records = MigrationRecords::for_intent(&intent);
        for phase in StoreMigrationPhase::ALL.iter().take(count) {
            execute_phase(&mut authority, *phase, &records)?;
        }
        drop(authority);
        match (
            count,
            FilesystemPlatformAdmission::reopen_unchecked_for_tests(store.path()),
        ) {
            (0, Ok(admission)) => {
                let authority =
                    super::FilesystemStoreMigrationAuthority::open(admission, maximum_policy())?;
                authority.verify_current(&intent)?;
            }
            (_, Err(FilesystemPlatformAdmissionError::Namespace { source })) if count > 0 => {
                assert_eq!(source.kind(), io::ErrorKind::InvalidData, "prefix {count}");
            }
            (_, result) => {
                return Err(format!(
                    "prefix {count}: incorrect v1 authority outcome: {}",
                    result
                        .err()
                        .map_or_else(|| "admitted".to_owned(), |error| format!("{error:?}"))
                )
                .into());
            }
        }
        assert_eq!(
            version_one_witness(store.path())?,
            before,
            "prefix {count}: v1 immutable bytes changed"
        );
        store.remove()?;
    }
    Ok(())
}
