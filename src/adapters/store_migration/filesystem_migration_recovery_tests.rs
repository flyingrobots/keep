//! Filesystem migration recovery laws: every forward prefix recovers.

use std::error::Error;
use std::fs;
use std::io;
use std::path::Path;

use super::filesystem_migration_test_fixture::{maximum_policy, open_authority};
use super::migration_resumption::{MigrationRecords, execute_phase};
use super::{
    AdmittedStoreFormatMarker, AdmittedStoreMigrationIntent, AdmittedStoreMigrationReceipt,
    FilesystemStoreMigrationAuthority, StoreMigrationFixedStage, StoreMigrationPhase,
    StoreMigrationRecoveryError, StoreMigrationRecoveryPlan, StoreMigrationStorage,
    recover_store_migration,
};

/// Runs the first `count` forward phases in-process, drops the writer, and
/// recovers under a fresh authority. Every prefix must end in exactly one
/// complete migration whose records admit, with every version-1 byte intact.
#[test]
fn every_forward_prefix_recovers_to_one_complete_migration() -> Result<(), Box<dyn Error>> {
    for count in 0..=StoreMigrationPhase::ALL.len() {
        let name = format!("filesystem-migration-recovery-prefix-{count}");
        let (sandbox, mut authority) = open_authority(&name)?;
        let intent = authority.observe_intent()?;
        let witness = version_one_witness(sandbox.path())?;
        StoreMigrationStorage::verify_current(&mut authority, &intent)?;
        let records = MigrationRecords::for_intent(&intent);
        for phase in StoreMigrationPhase::ALL.iter().take(count) {
            execute_phase(&mut authority, *phase, &records)?;
        }
        drop(authority);

        let mut recovered =
            FilesystemStoreMigrationAuthority::reopen_for_recovery_unchecked_for_tests(
                sandbox.path(),
                maximum_policy(),
            )?;
        let expected = recovered.observe_intent()?;
        let receipt = recover_store_migration(&mut recovered, &expected)
            .map_err(|error| format!("prefix {count}: {error}: {:?}", error.source()))?;
        match receipt.plan() {
            StoreMigrationRecoveryPlan::Resume { resume }
            | StoreMigrationRecoveryPlan::DiscardStage { resume, .. } => {
                assert_eq!(receipt.executed_phases().next(), Some(resume));
                assert_eq!(
                    receipt.executed_phases().last(),
                    Some(StoreMigrationPhase::SynchronizeRootAfterReceiptCleanup)
                );
            }
            _ => assert_eq!(receipt.executed_phases().count(), 0),
        }
        drop(recovered);

        match receipt.plan() {
            StoreMigrationRecoveryPlan::VersionOne => {
                assert_eq!(count, 0, "only an untouched store admits version one");
                assert!(!sandbox.path().join("migration.intent").exists());
            }
            StoreMigrationRecoveryPlan::Complete => {
                // The receipt is canonical from phase 20 on; the final root
                // synchronization leaves nothing a residue can observe.
                assert!(count >= 20, "prefix {count} reported complete");
            }
            StoreMigrationRecoveryPlan::Resume { .. } => {
                assert!(receipt.published().is_some(), "prefix {count} resumed");
                assert_complete_migration(sandbox.path(), &intent)?;
            }
            StoreMigrationRecoveryPlan::DiscardStage { .. } => {
                return Err(format!("prefix {count} needed a discard for an exact stage").into());
            }
        }
        if count > 0 {
            assert_complete_migration(sandbox.path(), &intent)?;
        }
        assert_eq!(
            version_one_witness(sandbox.path())?,
            witness,
            "prefix {count}"
        );
        sandbox.remove()?;
    }
    Ok(())
}

#[test]
fn a_truncated_intent_stage_is_discarded_and_the_migration_completes() -> Result<(), Box<dyn Error>>
{
    let (sandbox, mut authority) = open_authority("filesystem-migration-recovery-truncated")?;
    let intent = authority.observe_intent()?;
    StoreMigrationStorage::verify_current(&mut authority, &intent)?;
    StoreMigrationStorage::write_intent_stage(&mut authority, &intent)?;
    drop(authority);
    let stage = sandbox.path().join("migration.intent.next");
    let mut bytes = fs::read(&stage)?;
    bytes.truncate(100);
    fs::write(&stage, &bytes)?;

    let mut recovered = FilesystemStoreMigrationAuthority::reopen_for_recovery_unchecked_for_tests(
        sandbox.path(),
        maximum_policy(),
    )?;
    let expected = recovered.observe_intent()?;
    let receipt = recover_store_migration(&mut recovered, &expected)?;
    drop(recovered);

    assert_eq!(
        receipt.plan(),
        StoreMigrationRecoveryPlan::DiscardStage {
            stage: StoreMigrationFixedStage::Intent,
            resume: StoreMigrationPhase::WriteIntentStage,
        }
    );
    assert_complete_migration(sandbox.path(), &intent)?;
    sandbox.remove()?;
    Ok(())
}

#[test]
fn a_corrupt_durable_intent_refuses_recovery_before_any_mutation() -> Result<(), Box<dyn Error>> {
    let (sandbox, mut authority) = open_authority("filesystem-migration-recovery-corrupt")?;
    let intent = authority.observe_intent()?;
    StoreMigrationStorage::verify_current(&mut authority, &intent)?;
    let records = MigrationRecords::for_intent(&intent);
    for phase in StoreMigrationPhase::ALL.iter().take(6) {
        execute_phase(&mut authority, *phase, &records)?;
    }
    drop(authority);
    let canonical = sandbox.path().join("migration.intent");
    let mut bytes = fs::read(&canonical)?;
    let last = bytes.last_mut().ok_or("intent is empty")?;
    *last ^= 1;
    fs::write(&canonical, &bytes)?;
    let before = fs::read_dir(sandbox.path())?.count();

    let mut recovered = FilesystemStoreMigrationAuthority::reopen_for_recovery_unchecked_for_tests(
        sandbox.path(),
        maximum_policy(),
    )?;
    let expected = recovered.observe_intent()?;
    let error = recover_store_migration(&mut recovered, &expected)
        .err()
        .ok_or("a corrupt durable intent was recovered")?;
    drop(recovered);

    assert!(matches!(
        error,
        StoreMigrationRecoveryError::Ambiguity { .. }
    ));
    assert_eq!(fs::read_dir(sandbox.path())?.count(), before);
    assert!(!sandbox.path().join("reader.lock").exists());
    sandbox.remove()?;
    Ok(())
}

pub(super) fn assert_complete_migration(
    root: &Path,
    intent: &super::CanonicalStoreMigrationIntent,
) -> Result<(), Box<dyn Error>> {
    let intent_bytes = fs::read(root.join("migration.intent"))?;
    let marker_bytes = fs::read(root.join("FORMAT"))?;
    let receipt_bytes = fs::read(root.join("migration.receipt"))?;
    let admitted = AdmittedStoreMigrationIntent::decode(&intent_bytes)?;
    let marker = AdmittedStoreFormatMarker::decode(&marker_bytes)?;
    let _receipt = AdmittedStoreMigrationReceipt::decode(&receipt_bytes, &admitted, &marker)?;
    assert_eq!(admitted.encoded(), intent.encoded());
    for stage in [
        "migration.intent.next",
        "FORMAT.next",
        "migration.receipt.next",
    ] {
        assert!(!root.join(stage).exists(), "{stage} survived recovery");
    }
    for directory in [
        "retention/roots",
        "retention/manifests",
        "gc",
        "recovery/dispositions",
    ] {
        assert!(root.join(directory).is_dir(), "{directory} is absent");
    }
    assert_eq!(fs::metadata(root.join("reader.lock"))?.len(), 0);
    Ok(())
}

pub(super) fn version_one_witness(root: &Path) -> io::Result<Vec<(String, Vec<u8>)>> {
    let mut witness = vec![("HEAD".to_owned(), fs::read(root.join("HEAD"))?)];
    for pool in ["segments", "catalogs"] {
        for entry in fs::read_dir(root.join(pool))? {
            let entry = entry?;
            let name = format!("{pool}/{}", entry.file_name().to_string_lossy());
            witness.push((name, fs::read(entry.path())?));
        }
    }
    witness.sort();
    Ok(witness)
}
