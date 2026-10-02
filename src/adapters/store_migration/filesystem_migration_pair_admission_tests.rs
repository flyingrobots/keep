//! This module owns refusal of substituted stage/canonical pairs before resumption.

use std::error::Error;
use std::fs;

use super::filesystem_migration_recovery_tests::version_one_witness;
use super::filesystem_migration_test_fixture::{maximum_policy, open_authority};
use super::migration_resumption::{MigrationRecords, execute_phase};
use super::{
    FilesystemStoreMigrationAuthority, StoreMigrationPhase, StoreMigrationRecoveryError,
    StoreMigrationStorage, recover_store_migration,
};
use crate::adapters::filesystem_exact_record::{ExactRecordError, ExactRecordRefusal};

#[test]
fn substituted_canonical_pairs_refuse_during_adoption_before_forward_execution()
-> Result<(), Box<dyn Error>> {
    for (count, canonical, stage) in [
        (3, "migration.intent", "migration.intent.next"),
        (12, "FORMAT", "FORMAT.next"),
        (18, "migration.receipt", "migration.receipt.next"),
    ] {
        refuse_pair(count, canonical, stage)?;
    }
    Ok(())
}

fn refuse_pair(count: usize, canonical: &str, stage: &str) -> Result<(), Box<dyn Error>> {
    let (sandbox, mut authority) = open_authority("migration-pair-admission")?;
    let intent = authority.observe_intent()?;
    StoreMigrationStorage::verify_current(&mut authority, &intent)?;
    let records = MigrationRecords::for_intent(&intent);
    for phase in StoreMigrationPhase::ALL.iter().take(count) {
        execute_phase(&mut authority, *phase, &records)?;
    }
    drop(authority);
    let bytes = fs::read(sandbox.path().join(canonical))?;
    fs::remove_file(sandbox.path().join(canonical))?;
    fs::write(sandbox.path().join(canonical), &bytes)?;
    let witness = version_one_witness(sandbox.path())?;
    let mut recovered = FilesystemStoreMigrationAuthority::reopen_for_recovery_unchecked_for_tests(
        sandbox.path(),
        maximum_policy(),
    )?;
    let expected = recovered.observe_intent()?;
    let error = recover_store_migration(&mut recovered, &expected)
        .err()
        .ok_or("a substituted pair was admitted")?;
    let StoreMigrationRecoveryError::Adoption { source } = error else {
        return Err(format!("{canonical}: substitution reached a forward phase: {error}").into());
    };
    assert!(matches!(
        source
            .get_ref()
            .and_then(|error| error.downcast_ref::<ExactRecordError>()),
        Some(ExactRecordError::Refused(
            ExactRecordRefusal::KindLengthOrIdentity
        ))
    ));
    assert_eq!(fs::read(sandbox.path().join(canonical))?, bytes);
    assert_eq!(fs::read(sandbox.path().join(stage))?, bytes);
    assert_eq!(version_one_witness(sandbox.path())?, witness);
    drop(recovered);
    sandbox.remove()?;
    Ok(())
}
