//! Version-2 store-migration execution laws.

#[path = "store_migration_storage/recording_storage.rs"]
pub mod recording_storage;
mod support;

use std::error::Error;
use std::io;

use keep::{
    AdmittedStoreMigrationIntent, CanonicalStoreFormatMarker, CanonicalStoreMigrationIntent,
    CanonicalStoreMigrationReceipt, StoreMigrationError, StoreMigrationPhase,
    execute_store_migration,
};
use recording_storage::RecordingStorage;

const INTENT: &str = include_str!("../conformance/segment-store/v2/migration-intent.hex");

#[test]
fn migration_executes_every_phase_before_returning_its_receipt() -> Result<(), Box<dyn Error>> {
    let (intent, marker) = artifacts()?;
    let expected = CanonicalStoreMigrationReceipt::from_canonical(&intent, &marker);
    let mut storage = RecordingStorage::default();

    let receipt = execute_store_migration(&mut storage, &intent)?;

    assert_eq!(receipt, expected);
    assert_eq!(storage.verification_count(), 1);
    assert_eq!(storage.observed(), StoreMigrationPhase::ALL);
    Ok(())
}

#[test]
fn current_verification_refuses_before_every_migration_phase() -> Result<(), Box<dyn Error>> {
    let (intent, _marker) = artifacts()?;
    let mut storage = RecordingStorage::verification_failure();

    let Err(error) = execute_store_migration(&mut storage, &intent) else {
        return Err("current-state refusal unexpectedly admitted migration".into());
    };

    match error {
        StoreMigrationError::CurrentVerification { source } => {
            assert_eq!(source.kind(), io::ErrorKind::PermissionDenied);
        }
        other @ StoreMigrationError::Storage { .. } => {
            return Err(format!("unexpected migration error: {other}").into());
        }
    }
    assert_eq!(storage.verification_count(), 1);
    assert!(storage.observed().is_empty());
    Ok(())
}

#[test]
fn every_phase_failure_stops_before_all_later_mutation() -> Result<(), Box<dyn Error>> {
    let (intent, _marker) = artifacts()?;
    for (index, phase) in StoreMigrationPhase::ALL.into_iter().enumerate() {
        let mut storage = RecordingStorage::failing_at(phase);
        let Err(error) = execute_store_migration(&mut storage, &intent) else {
            return Err("injected refusal unexpectedly admitted migration".into());
        };
        assert_storage_error(error, phase)?;
        assert_eq!(storage.verification_count(), 1);
        let expected = StoreMigrationPhase::ALL
            .get(..=index)
            .ok_or("migration phase prefix is out of bounds")?;
        assert_eq!(storage.observed(), expected);
    }
    Ok(())
}

fn assert_storage_error(
    error: StoreMigrationError,
    expected_phase: StoreMigrationPhase,
) -> Result<(), Box<dyn Error>> {
    match error {
        StoreMigrationError::Storage { phase, source } => {
            assert_eq!(phase, expected_phase);
            assert_eq!(source.kind(), io::ErrorKind::Other);
            Ok(())
        }
        other @ StoreMigrationError::CurrentVerification { .. } => {
            Err(format!("unexpected migration error: {other}").into())
        }
    }
}

fn artifacts() -> Result<(CanonicalStoreMigrationIntent, CanonicalStoreFormatMarker), Box<dyn Error>>
{
    let intent_bytes = support::decode_hex(INTENT.trim_end())?;
    let admitted = AdmittedStoreMigrationIntent::decode(&intent_bytes)?;
    Ok((
        CanonicalStoreMigrationIntent::from_admitted(&admitted),
        CanonicalStoreFormatMarker::version_two(),
    ))
}
