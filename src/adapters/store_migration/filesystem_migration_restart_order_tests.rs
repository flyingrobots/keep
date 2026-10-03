//! Filesystem restart refusal at impossible migration orderings.

use super::filesystem_migration_restart_test_fixture::{prefix, refusal};
use super::{
    StoreMigrationEffect as Effect, StoreMigrationFixedStage as Stage,
    StoreMigrationPhase as Phase, StoreMigrationRecoveryAmbiguity as Ambiguity,
    StoreMigrationRecoveryError as Recovery,
};
use std::{error::Error, fs};

// Size: medium. Oracle: no namespace, marker or receipt effect may precede durable intent.
// Delete only if the migration ordering contract disappears or stronger coverage subsumes it.
#[test]
fn effects_without_durable_intent_preserve_restart_evidence() -> Result<(), Box<dyn Error>> {
    for (name, effect) in [
        ("reader.lock", Effect::Namespace),
        ("FORMAT", Effect::Marker),
        ("migration.receipt", Effect::Receipt),
    ] {
        let store = prefix(
            &format!("restart-before-intent-{name}"),
            Phase::WriteIntentStage,
        )?;
        fs::write(store.path().join(name), [])?;
        let error = refusal(store.path())?;
        assert!(
            matches!(error.downcast_ref::<Recovery>(), Some(Recovery::Ambiguity {
            source: Ambiguity::EffectBeforeIntent { effect: observed }
        }) if *observed == effect),
            "{name}: {error:?}"
        );
        store.remove()?;
    }
    Ok(())
}

// Size: medium. Oracle: later effects require cleanup of the previous stage.
// Delete only if the cleanup ordering requirement disappears.
#[test]
fn stages_surviving_later_effects_refuse_restart() -> Result<(), Box<dyn Error>> {
    for (phase, name, stage, effect) in [
        (
            Phase::LinkIntent,
            "reader.lock",
            Stage::Intent,
            Effect::Namespace,
        ),
        (Phase::LinkIntent, "FORMAT", Stage::Intent, Effect::Marker),
        (
            Phase::LinkIntent,
            "migration.receipt",
            Stage::Intent,
            Effect::Receipt,
        ),
        (
            Phase::LinkMarker,
            "migration.receipt",
            Stage::Marker,
            Effect::Receipt,
        ),
    ] {
        let store = prefix(&format!("restart-stage-after-{phase:?}-{name}"), phase)?;
        fs::write(store.path().join(name), [])?;
        let error = refusal(store.path())?;
        assert!(
            matches!(error.downcast_ref::<Recovery>(), Some(Recovery::Ambiguity {
            source: Ambiguity::StageAfterEffect { stage: observed_stage, effect: observed_effect }
        }) if *observed_stage == stage && *observed_effect == effect),
            "{name}: {error:?}"
        );
        store.remove()?;
    }
    Ok(())
}

// Size: medium. Oracle: filesystem-realizable holes cannot be filled over later names.
// Delete only if the ordered namespace prefix is removed.
#[test]
fn namespace_holes_refuse_restart_at_the_exact_coordinates() -> Result<(), Box<dyn Error>> {
    for (name, absent, present) in [
        ("reader.lock", 0, 1),
        ("retention", 1, 4),
        ("retention/roots", 2, 3),
        ("retention/manifests", 3, 4),
        ("gc", 4, 5),
    ] {
        let store = prefix(
            &format!("restart-namespace-hole-{absent}"),
            Phase::AdmitNamespacePrefix,
        )?;
        let path = store.path().join(name);
        if path.is_dir() {
            fs::remove_dir_all(path)?;
        } else {
            fs::remove_file(path)?;
        }
        let error = refusal(store.path())?;
        assert!(
            matches!(error.downcast_ref::<Recovery>(), Some(Recovery::Ambiguity {
            source: Ambiguity::NamespaceOutOfOrder { absent: observed_absent, present: observed_present }
        }) if *observed_absent == absent && *observed_present == present),
            "{name}: {error:?}"
        );
        store.remove()?;
    }
    Ok(())
}

// Size: medium. Oracle: a marker cannot precede the complete namespace.
// Delete only if migration stops requiring the namespace before marker publication.
#[test]
fn a_marker_before_namespace_completion_preserves_evidence() -> Result<(), Box<dyn Error>> {
    let store = prefix("restart-marker-before-namespace", Phase::WriteMarkerStage)?;
    fs::remove_dir_all(store.path().join("recovery"))?;
    fs::remove_dir(store.path().join("gc"))?;
    let error = refusal(store.path())?;
    assert!(
        matches!(
            error.downcast_ref::<Recovery>(),
            Some(Recovery::Ambiguity {
                source: Ambiguity::MarkerBeforeNamespace
            })
        ),
        "{error:?}"
    );
    store.remove()?;
    Ok(())
}

// Size: medium. Oracle: a receipt requires the durable marker it binds.
// Delete only if receipts cease to bind the migration marker.
#[test]
fn a_receipt_without_its_marker_preserves_evidence() -> Result<(), Box<dyn Error>> {
    let store = prefix("restart-receipt-before-marker", Phase::WriteReceiptStage)?;
    fs::remove_file(store.path().join("FORMAT"))?;
    let error = refusal(store.path())?;
    assert!(
        matches!(
            error.downcast_ref::<Recovery>(),
            Some(Recovery::Ambiguity {
                source: Ambiguity::ReceiptBeforeMarker
            })
        ),
        "{error:?}"
    );
    store.remove()?;
    Ok(())
}
