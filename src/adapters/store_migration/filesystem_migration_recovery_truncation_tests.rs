//! This module owns exhaustive strict-stage-prefix recovery and preflight laws.

use std::error::Error;
use std::fs;

use super::filesystem_migration_recovery_tests::{assert_complete_migration, version_one_witness};
use super::filesystem_migration_test_fixture::{maximum_policy, open_authority};
use super::migration_resumption::{MigrationRecords, execute_phase};
use super::{
    FilesystemMigrationRecoveryRefusal, FilesystemStoreMigrationAuthority,
    StoreMigrationFixedStage, StoreMigrationPhase, StoreMigrationRecoveryError,
    StoreMigrationRecoveryPlan, StoreMigrationStorage, recover_store_migration,
};

#[test]
fn every_strict_fixed_stage_prefix_is_discarded_without_changing_version_one_bytes()
-> Result<(), Box<dyn Error>> {
    for (stage, phase, name, length) in [
        (
            StoreMigrationFixedStage::Intent,
            StoreMigrationPhase::WriteIntentStage,
            "migration.intent.next",
            super::MIGRATION_INTENT_LENGTH,
        ),
        (
            StoreMigrationFixedStage::Marker,
            StoreMigrationPhase::WriteMarkerStage,
            "FORMAT.next",
            super::FORMAT_MARKER_LENGTH,
        ),
        (
            StoreMigrationFixedStage::Receipt,
            StoreMigrationPhase::WriteReceiptStage,
            "migration.receipt.next",
            super::MIGRATION_RECEIPT_LENGTH,
        ),
    ] {
        for length in 0..length {
            recover_truncation(stage, phase, name, length)?;
        }
    }
    Ok(())
}

fn recover_truncation(
    stage: StoreMigrationFixedStage,
    phase: StoreMigrationPhase,
    name: &str,
    length: usize,
) -> Result<(), Box<dyn Error>> {
    let (sandbox, mut authority) = open_authority("migration-all-truncations")?;
    let intent = authority.observe_intent()?;
    let witness = version_one_witness(sandbox.path())?;
    StoreMigrationStorage::verify_current(&mut authority, &intent)?;
    let records = MigrationRecords::for_intent(&intent);
    for current in StoreMigrationPhase::ALL {
        execute_phase(&mut authority, current, &records)?;
        if current == phase {
            break;
        }
    }
    drop(authority);
    fs::OpenOptions::new()
        .write(true)
        .open(sandbox.path().join(name))?
        .set_len(u64::try_from(length)?)?;
    let mut recovered = FilesystemStoreMigrationAuthority::reopen_for_recovery_unchecked_for_tests(
        sandbox.path(),
        maximum_policy(),
    )?;
    let expected = recovered.observe_intent()?;
    let receipt = recover_store_migration(&mut recovered, &expected)?;
    assert_eq!(
        receipt.plan(),
        StoreMigrationRecoveryPlan::DiscardStage {
            stage,
            resume: phase
        }
    );
    drop(recovered);
    assert_complete_migration(sandbox.path(), &intent)?;
    assert_eq!(version_one_witness(sandbox.path())?, witness);
    sandbox.remove()?;
    Ok(())
}

#[test]
fn unexpected_nested_residue_refuses_before_creating_a_marker_stage() -> Result<(), Box<dyn Error>>
{
    let (sandbox, mut authority) = open_authority("migration-nested-refusal")?;
    let intent = authority.observe_intent()?;
    StoreMigrationStorage::verify_current(&mut authority, &intent)?;
    let records = MigrationRecords::for_intent(&intent);
    for phase in StoreMigrationPhase::ALL.iter().take(8) {
        execute_phase(&mut authority, *phase, &records)?;
    }
    drop(authority);
    fs::write(sandbox.path().join("gc/unexpected"), b"preserve")?;
    let witness = version_one_witness(sandbox.path())?;
    let mut recovered = FilesystemStoreMigrationAuthority::reopen_for_recovery_unchecked_for_tests(
        sandbox.path(),
        maximum_policy(),
    )?;
    let expected = recovered.observe_intent()?;
    let error = recover_store_migration(&mut recovered, &expected)
        .err()
        .ok_or("unexpected nested residue admitted")?;
    let StoreMigrationRecoveryError::Adoption { source } = error else {
        return Err("nested residue was not rejected before adoption".into());
    };
    assert!(matches!(
        source
            .get_ref()
            .and_then(|error| error.downcast_ref::<FilesystemMigrationRecoveryRefusal>()),
        Some(FilesystemMigrationRecoveryRefusal::NamespacePreflight { .. })
    ));
    assert!(!sandbox.path().join("FORMAT.next").exists());
    assert_eq!(fs::read(sandbox.path().join("gc/unexpected"))?, b"preserve");
    assert_eq!(version_one_witness(sandbox.path())?, witness);
    drop(recovered);
    sandbox.remove()?;
    Ok(())
}
