//! Restart laws for fixed-stage identity and contradictory paired evidence.

use super::filesystem_migration_restart_test_fixture::{prefix, refusal};
use super::{
    StoreMigrationFixedStage as Stage, StoreMigrationPhase as Phase,
    StoreMigrationRecoveryAmbiguity as Ambiguity, StoreMigrationRecoveryError as Recovery,
};
use crate::adapters::filesystem_exact_record::{ExactRecordError, ExactRecordRefusal};
use std::{error::Error, fs};

// Size: medium. Oracle: stage and canonical record must share one inode, not only bytes.
// Delete only if the fixed-stage protocol disappears or stronger restart laws subsume it.
#[test]
fn byte_equal_stage_substitution_refuses_before_restart_effects() -> Result<(), Box<dyn Error>> {
    for (name, phase) in [
        ("migration.intent.next", Phase::LinkIntent),
        ("FORMAT.next", Phase::LinkMarker),
        ("migration.receipt.next", Phase::LinkReceipt),
    ] {
        let store = prefix(&format!("restart-substitution-{name}"), phase)?;
        let path = store.path().join(name);
        let bytes = fs::read(&path)?;
        fs::remove_file(&path)?;
        fs::write(path, bytes)?;
        let error = refusal(store.path())?;
        assert!(
            matches!(error.downcast_ref::<Recovery>(), Some(Recovery::Adoption { source })
            if matches!(source.get_ref().and_then(|source| source.downcast_ref::<ExactRecordError>()),
            Some(ExactRecordError::Refused(ExactRecordRefusal::KindLengthOrIdentity)))),
            "{name}: {error:?}"
        );
        store.remove()?;
    }
    Ok(())
}

// Size: medium. Oracle: contradictory stage/target bytes refuse rather than choose a winner.
// Delete only if paired migration evidence is no longer part of the protocol.
#[test]
fn conflicting_stage_bytes_preserve_both_records_on_restart() -> Result<(), Box<dyn Error>> {
    for (name, phase, stage) in [
        ("migration.intent.next", Phase::LinkIntent, Stage::Intent),
        ("FORMAT.next", Phase::LinkMarker, Stage::Marker),
        ("migration.receipt.next", Phase::LinkReceipt, Stage::Receipt),
    ] {
        let store = prefix(&format!("restart-conflict-{name}"), phase)?;
        let path = store.path().join(name);
        let mut bytes = fs::read(&path)?;
        *bytes.last_mut().ok_or("empty stage")? ^= 1;
        fs::remove_file(&path)?;
        fs::write(path, bytes)?;
        let error = refusal(store.path())?;
        assert!(
            matches!(error.downcast_ref::<Recovery>(), Some(Recovery::Ambiguity {
            source: Ambiguity::StageDiffers { stage: observed }
        }) if *observed == stage),
            "{name}: {error:?}"
        );
        store.remove()?;
    }
    Ok(())
}
