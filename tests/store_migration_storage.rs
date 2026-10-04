//! Version-2 store-migration storage capability laws.

#[path = "store_migration_storage/recording_storage.rs"]
pub mod recording_storage;
mod support;

use std::error::Error;
use std::io;

use keep::{
    AdmittedStoreMigrationIntent, CanonicalStoreFormatMarker, CanonicalStoreMigrationIntent,
    CanonicalStoreMigrationReceipt, StoreMigrationPhase, StoreMigrationStorage,
};
use recording_storage::RecordingStorage;

const INTENT: &str = include_str!("../conformance/segment-store/v2/migration-intent.hex");

#[test]
fn storage_port_names_every_migration_phase() -> Result<(), Box<dyn Error>> {
    let intent_bytes = support::decode_hex(INTENT.trim_end())?;
    let admitted = AdmittedStoreMigrationIntent::decode(&intent_bytes)?;
    let intent = CanonicalStoreMigrationIntent::from_admitted(&admitted);
    let marker = CanonicalStoreFormatMarker::version_two();
    let receipt = CanonicalStoreMigrationReceipt::from_canonical(&intent, &marker);
    let mut storage = RecordingStorage::default();

    exercise_storage(&mut storage, &intent, &marker, &receipt)?;

    assert_eq!(storage.verification_count(), 1);
    assert_eq!(storage.observed(), StoreMigrationPhase::ALL);
    Ok(())
}

fn exercise_storage(
    storage: &mut impl StoreMigrationStorage,
    intent: &CanonicalStoreMigrationIntent,
    marker: &CanonicalStoreFormatMarker,
    receipt: &CanonicalStoreMigrationReceipt,
) -> io::Result<()> {
    storage.verify_current(intent)?;
    exercise_intent(storage, intent)?;
    exercise_namespace(storage)?;
    exercise_marker(storage, marker)?;
    exercise_receipt(storage, receipt)
}

fn exercise_intent(
    storage: &mut impl StoreMigrationStorage,
    intent: &CanonicalStoreMigrationIntent,
) -> io::Result<()> {
    storage.write_intent_stage(intent)?;
    storage.synchronize_intent_stage()?;
    storage.link_intent(intent)?;
    storage.synchronize_root_after_intent()?;
    storage.remove_intent_stage()?;
    storage.synchronize_root_after_intent_cleanup()
}

fn exercise_namespace(storage: &mut impl StoreMigrationStorage) -> io::Result<()> {
    storage.admit_reader_fence()?;
    storage.admit_namespace_prefix()?;
    storage.synchronize_root_after_namespace()
}

fn exercise_marker(
    storage: &mut impl StoreMigrationStorage,
    marker: &CanonicalStoreFormatMarker,
) -> io::Result<()> {
    storage.write_marker_stage(marker)?;
    storage.synchronize_marker_stage()?;
    storage.link_marker(marker)?;
    storage.synchronize_root_after_marker()?;
    storage.remove_marker_stage()?;
    storage.synchronize_root_after_marker_cleanup()
}

fn exercise_receipt(
    storage: &mut impl StoreMigrationStorage,
    receipt: &CanonicalStoreMigrationReceipt,
) -> io::Result<()> {
    storage.write_receipt_stage(receipt)?;
    storage.synchronize_receipt_stage()?;
    storage.link_receipt(receipt)?;
    storage.synchronize_root_after_receipt()?;
    storage.remove_receipt_stage()?;
    storage.synchronize_root_after_receipt_cleanup()
}
