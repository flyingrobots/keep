//! This module owns the law that recovery receipts preserve observed evidence.

use std::error::Error;
use std::fs;

use super::filesystem_migration_test_fixture::{maximum_policy, open_authority};
use super::migration_resumption::{MigrationRecords, execute_phase};
use super::{
    FilesystemStoreMigrationAuthority, StoreMigrationPhase, StoreMigrationStorage,
    recover_store_migration,
};

const DIRECTORIES: [&str; 6] = [
    "retention",
    "retention/roots",
    "retention/manifests",
    "gc",
    "recovery",
    "recovery/dispositions",
];

#[test]
fn recovery_receipts_distinguish_each_observed_namespace_prefix() -> Result<(), Box<dyn Error>> {
    let (sandbox, mut authority) = open_authority("migration-receipt-prefix-evidence")?;
    let intent = authority.observe_intent()?;
    StoreMigrationStorage::verify_current(&mut authority, &intent)?;
    let records = MigrationRecords::for_intent(&intent);
    for phase in StoreMigrationPhase::ALL.iter().take(7) {
        execute_phase(&mut authority, *phase, &records)?;
    }
    drop(authority);
    let mut previous = None;
    for count in 0..=DIRECTORIES.len() {
        for name in DIRECTORIES.iter().take(count) {
            fs::create_dir(sandbox.path().join(name))?;
        }
        let mut recovered =
            FilesystemStoreMigrationAuthority::reopen_for_recovery_unchecked_for_tests(
                sandbox.path(),
                maximum_policy(),
            )?;
        let expected = recovered.observe_intent()?;
        let receipt = recover_store_migration(&mut recovered, &expected)?;
        assert_eq!(receipt.intent_digest(), Some(intent.digest()));
        assert_eq!(
            receipt.observed_namespace_prefix().len(),
            count.checked_add(1).ok_or("prefix overflow")?
        );
        let names: Vec<_> = std::iter::once("reader.lock")
            .chain(DIRECTORIES.into_iter().take(count))
            .collect();
        assert_eq!(
            receipt
                .observed_namespace_prefix()
                .names()
                .collect::<Vec<_>>(),
            names
        );
        if let Some(previous) = previous {
            assert_ne!(
                receipt, previous,
                "distinct observed prefixes collapsed at {count}"
            );
        }
        previous = Some(receipt);
        let complete = recover_store_migration(&mut recovered, &expected)?;
        assert_eq!(complete.intent_digest(), Some(intent.digest()));
        assert_eq!(complete.observed_namespace_prefix().len(), 7);
        assert_eq!(complete.executed_phases().count(), 0);
        drop(recovered);
        fs::remove_file(sandbox.path().join("FORMAT"))?;
        fs::remove_file(sandbox.path().join("migration.receipt"))?;
        for name in DIRECTORIES.iter().rev() {
            fs::remove_dir(sandbox.path().join(name))?;
        }
    }
    sandbox.remove()?;
    Ok(())
}
