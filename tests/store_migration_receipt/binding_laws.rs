//! This module owns migration-receipt integrity and binding laws.

use keep::{
    AdmittedStoreFormatMarker, AdmittedStoreMigrationIntent, AdmittedStoreMigrationReceipt,
    StoreMigrationReceiptDecodeError,
};

use super::fixture::{
    DISPOSITION_DIGEST, INITIAL_GC_DIGEST, INITIAL_RETENTION_DIGEST, intent_bytes, marker_bytes,
    receipt_bytes,
};
use super::harness::{
    assert_semantic_refusal, decode_receipt, digest_intent, flip_byte, mutated_array,
    refresh_checksum,
};

#[test]
fn receipt_integrity_and_binding_have_exact_precedence() -> Result<(), Box<dyn std::error::Error>> {
    let mut checksum = receipt_bytes()?;
    flip_byte(&mut checksum, 24)?;
    assert!(matches!(
        decode_receipt(&checksum)?,
        Err(StoreMigrationReceiptDecodeError::ChecksumMismatch { .. })
    ));

    let intent = intent_bytes()?;
    let marker = marker_bytes()?;
    assert_semantic_refusal(
        24,
        StoreMigrationReceiptDecodeError::IntentDigestMismatch {
            expected: digest_intent(&intent),
            observed: mutated_array(&receipt_bytes()?, 24, 0)?,
        },
    )?;
    assert_semantic_refusal(
        56,
        StoreMigrationReceiptDecodeError::StoreIdentifierMismatch {
            expected: *AdmittedStoreMigrationIntent::decode(&intent)?
                .store_identifier()
                .as_bytes(),
            observed: mutated_array(&receipt_bytes()?, 56, 0)?,
        },
    )?;
    assert_semantic_refusal(
        88,
        StoreMigrationReceiptDecodeError::FormatMarkerDigestMismatch {
            expected: *AdmittedStoreFormatMarker::decode(&marker)?
                .digest()
                .as_bytes(),
            observed: mutated_array(&receipt_bytes()?, 88, 0)?,
        },
    )?;
    assert_semantic_refusal(
        120,
        StoreMigrationReceiptDecodeError::InitialRetentionStateDigestMismatch {
            expected: INITIAL_RETENTION_DIGEST,
            observed: mutated_array(&receipt_bytes()?, 120, 0)?,
        },
    )?;
    assert_semantic_refusal(
        152,
        StoreMigrationReceiptDecodeError::InitialGcStateDigestMismatch {
            expected: INITIAL_GC_DIGEST,
            observed: mutated_array(&receipt_bytes()?, 152, 0)?,
        },
    )?;
    assert_semantic_refusal(
        184,
        StoreMigrationReceiptDecodeError::EmptyDispositionSetDigestMismatch {
            expected: DISPOSITION_DIGEST,
            observed: mutated_array(&receipt_bytes()?, 184, 0)?,
        },
    )?;
    assert_semantic_refusal(
        221,
        StoreMigrationReceiptDecodeError::UnsupportedSynchronizationBits {
            supported: 0x03ff,
            observed: 0x0001_03ff,
        },
    )?;
    assert_semantic_refusal(
        222,
        StoreMigrationReceiptDecodeError::IncompleteSynchronizationMask {
            required: 0x03ff,
            observed: 0x02ff,
        },
    )?;

    let mut alternative_intent = intent;
    flip_byte(&mut alternative_intent, 159)?;
    refresh_checksum(
        &mut alternative_intent,
        224,
        b"keep.store-migration-intent-checksum/v2\0",
    )?;
    let alternative = AdmittedStoreMigrationIntent::decode(&alternative_intent)?;
    let receipt = receipt_bytes()?;
    let marker = AdmittedStoreFormatMarker::decode(&marker)?;
    assert!(matches!(
        AdmittedStoreMigrationReceipt::decode(&receipt, &alternative, &marker),
        Err(StoreMigrationReceiptDecodeError::IntentDigestMismatch { .. })
    ));
    Ok(())
}
