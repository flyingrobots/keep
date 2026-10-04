//! Canonical version-2 store-migration receipt laws.

#[path = "store_migration_receipt/binding_laws.rs"]
mod binding_laws;
#[path = "store_migration_receipt/fixture.rs"]
mod fixture;
#[path = "store_migration_receipt/harness.rs"]
mod harness;
mod support;

use fixture::{
    DISPOSITION_DIGEST, INITIAL_GC_DIGEST, INITIAL_RETENTION_DIGEST, intent_bytes, marker_bytes,
    receipt_bytes,
};
use harness::{assert_fixed_refusal, assert_receipt_refusal, mutated_array};
use keep::{
    AdmittedStoreFormatMarker, AdmittedStoreMigrationIntent, AdmittedStoreMigrationReceipt,
    StoreMigrationReceiptDecodeError,
};

#[test]
fn receipt_admits_every_frozen_completion_coordinate() -> Result<(), Box<dyn std::error::Error>> {
    let intent_bytes = intent_bytes()?;
    let marker_bytes = marker_bytes()?;
    let receipt_bytes = receipt_bytes()?;
    let intent = AdmittedStoreMigrationIntent::decode(&intent_bytes)?;
    let marker = AdmittedStoreFormatMarker::decode(&marker_bytes)?;
    let receipt = AdmittedStoreMigrationReceipt::decode(&receipt_bytes, &intent, &marker)?;

    assert_eq!(receipt.encoded(), receipt_bytes);
    assert_eq!(receipt.intent_digest(), intent.digest());
    assert_eq!(receipt.store_identifier(), intent.store_identifier());
    assert_eq!(receipt.format_marker_digest(), marker.digest());
    assert_eq!(
        receipt.initial_retention_state_digest().as_bytes(),
        &INITIAL_RETENTION_DIGEST
    );
    assert_eq!(
        receipt.initial_gc_state_digest().as_bytes(),
        &INITIAL_GC_DIGEST
    );
    assert_eq!(
        receipt.empty_disposition_set_digest().as_bytes(),
        &DISPOSITION_DIGEST
    );
    assert_eq!(receipt.synchronization_mask().bits(), 0x03ff);
    Ok(())
}

#[test]
fn receipt_framing_has_exact_first_refusals() -> Result<(), Box<dyn std::error::Error>> {
    let bytes = receipt_bytes()?;
    let mut truncated = bytes.clone();
    assert!(truncated.pop().is_some());
    assert_receipt_refusal(
        &truncated,
        StoreMigrationReceiptDecodeError::WrongLength {
            expected: 256,
            observed: 255,
        },
    )?;
    let mut extended = bytes.clone();
    extended.push(0);
    assert_receipt_refusal(
        &extended,
        StoreMigrationReceiptDecodeError::WrongLength {
            expected: 256,
            observed: 257,
        },
    )?;
    assert_fixed_refusal(
        0,
        StoreMigrationReceiptDecodeError::InvalidMagic {
            observed: mutated_array(&bytes, 0, 0)?,
        },
    )?;
    assert_fixed_refusal(
        17,
        StoreMigrationReceiptDecodeError::UnsupportedVersion {
            expected: 2,
            observed: 3,
        },
    )?;
    assert_fixed_refusal(
        19,
        StoreMigrationReceiptDecodeError::InvalidRecordLength {
            expected: 256,
            observed: 257,
        },
    )?;
    assert_fixed_refusal(
        23,
        StoreMigrationReceiptDecodeError::UnsupportedFlags { observed: 1 },
    )?;
    Ok(())
}
