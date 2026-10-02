//! Exact restart diagnostics and preserved filesystem evidence for damaged records.

use super::filesystem_migration_restart_test_fixture::{prefix, refusal};
use super::{
    StoreFormatMarkerDecodeError as MarkerDecode, StoreMigrationFixedStage as Stage,
    StoreMigrationIntentDecodeError as IntentDecode, StoreMigrationPhase as Phase,
    StoreMigrationReceiptDecodeError as ReceiptDecode,
    StoreMigrationRecoveryAmbiguity as Ambiguity, StoreMigrationRecoveryError as Recovery,
    StoreMigrationStageDecodeError as StageDecode,
};
use std::{error::Error, fs};

// Size: medium. Oracle: the documented restart boundary retains the exact decoder
// checksum coordinates and preserves all names, inode identities and bytes.
// Delete only if migration records disappear or stronger restart laws subsume this matrix.
#[test]
fn checksum_damage_in_each_migration_record_preserves_restart_evidence()
-> Result<(), Box<dyn Error>> {
    for (name, phase) in [
        ("migration.intent.next", Phase::WriteIntentStage),
        ("migration.intent", Phase::RemoveIntentStage),
        ("FORMAT.next", Phase::WriteMarkerStage),
        ("FORMAT", Phase::RemoveMarkerStage),
        ("migration.receipt.next", Phase::WriteReceiptStage),
        ("migration.receipt", Phase::RemoveReceiptStage),
    ] {
        let store = prefix(&format!("restart-checksum-{name}"), phase)?;
        let path = store.path().join(name);
        let mut bytes = fs::read(&path)?;
        let start = bytes.len().checked_sub(32).ok_or("no record checksum")?;
        let expected: [u8; 32] = bytes.get(start..).ok_or("missing checksum")?.try_into()?;
        *bytes.last_mut().ok_or("empty record")? ^= 1;
        let observed: [u8; 32] = bytes.get(start..).ok_or("missing checksum")?.try_into()?;
        fs::write(path, bytes)?;
        let error = refusal(store.path())?;
        assert_eq!(
            checksum_coordinates(error.as_ref(), name),
            Some((expected, observed)),
            "{name}: {error:?}"
        );
        store.remove()?;
    }
    Ok(())
}

fn checksum_coordinates(error: &(dyn Error + 'static), name: &str) -> Option<([u8; 32], [u8; 32])> {
    let Recovery::Ambiguity { source } = error.downcast_ref::<Recovery>()? else {
        return None;
    };
    match (name, source) {
        (
            "migration.intent",
            Ambiguity::IntentUndecodable {
                source: IntentDecode::ChecksumMismatch { expected, observed },
            },
        )
        | (
            "FORMAT",
            Ambiguity::MarkerUndecodable {
                source: MarkerDecode::ChecksumMismatch { expected, observed },
            },
        )
        | (
            "migration.receipt",
            Ambiguity::ReceiptUndecodable {
                source: ReceiptDecode::ChecksumMismatch { expected, observed },
            },
        )
        | (
            "migration.intent.next",
            Ambiguity::StageUndecodable {
                stage: Stage::Intent,
                source:
                    StageDecode::Intent {
                        source: IntentDecode::ChecksumMismatch { expected, observed },
                    },
            },
        )
        | (
            "FORMAT.next",
            Ambiguity::StageUndecodable {
                stage: Stage::Marker,
                source:
                    StageDecode::Marker {
                        source: MarkerDecode::ChecksumMismatch { expected, observed },
                    },
            },
        )
        | (
            "migration.receipt.next",
            Ambiguity::StageUndecodable {
                stage: Stage::Receipt,
                source:
                    StageDecode::Receipt {
                        source: ReceiptDecode::ChecksumMismatch { expected, observed },
                    },
            },
        ) => Some((*expected, *observed)),
        _ => None,
    }
}

// Size: medium. Oracle: an overlong stage is not a discardable partial record.
// Delete only if stage framing disappears or stronger restart coverage subsumes it.
#[test]
fn overlong_stages_refuse_restart_without_disposal() -> Result<(), Box<dyn Error>> {
    for (name, phase, stage) in [
        (
            "migration.intent.next",
            Phase::WriteIntentStage,
            Stage::Intent,
        ),
        ("FORMAT.next", Phase::WriteMarkerStage, Stage::Marker),
        (
            "migration.receipt.next",
            Phase::WriteReceiptStage,
            Stage::Receipt,
        ),
    ] {
        let store = prefix(&format!("restart-overlong-{name}"), phase)?;
        let path = store.path().join(name);
        let mut bytes = fs::read(&path)?;
        bytes.push(0);
        let length = bytes.len();
        fs::write(path, bytes)?;
        let error = refusal(store.path())?;
        assert!(
            matches!(error.downcast_ref::<Recovery>(), Some(Recovery::Ambiguity {
            source: Ambiguity::StageOverlong { stage: observed_stage, observed }
        }) if *observed_stage == stage && *observed == length),
            "{name}: {error:?}"
        );
        store.remove()?;
    }
    Ok(())
}

// Size: medium. Oracle: a checksummed receipt from another root cannot bind this intent.
// Delete only if receipts no longer bind the migration intent.
#[test]
fn a_valid_foreign_receipt_refuses_its_conflicting_intent_digest() -> Result<(), Box<dyn Error>> {
    use super::AdmittedStoreMigrationIntent;
    let store = prefix("restart-foreign-receipt-local", Phase::RemoveReceiptStage)?;
    let foreign = prefix("restart-foreign-receipt-other", Phase::RemoveReceiptStage)?;
    let own_intent = fs::read(store.path().join("migration.intent"))?;
    let other_intent = fs::read(foreign.path().join("migration.intent"))?;
    let expected = AdmittedStoreMigrationIntent::decode(&own_intent)?.digest();
    let observed = AdmittedStoreMigrationIntent::decode(&other_intent)?.digest();
    fs::copy(
        foreign.path().join("migration.receipt"),
        store.path().join("migration.receipt"),
    )?;
    let error = refusal(store.path())?;
    assert!(
        matches!(error.downcast_ref::<Recovery>(), Some(Recovery::Ambiguity {
        source: Ambiguity::ReceiptUndecodable { source: ReceiptDecode::IntentDigestMismatch { expected: found_expected, observed: found_observed } }
    }) if found_expected == expected.as_bytes() && found_observed == observed.as_bytes()),
        "{error:?}"
    );
    store.remove()?;
    foreign.remove()?;
    Ok(())
}
