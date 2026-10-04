//! This module owns truthful GC execution failure and reopened-state laws.

use super::{LIVE_SEGMENT_NAME, ORPHAN_SEGMENT_NAME, authority, disposed_store, gc_entries, plan};
use crate::adapters::gc::{
    FilesystemGcError, GcExecutionError, GcExecutionPhase as Phase, GcExecutionPoint,
    GcRecoveryPlan,
};
use crate::adapters::retention::{
    RetentionEffectDurability as Durability, RetentionNamespaceEffect as Effect,
    RetentionStorageBoundary as Boundary,
};
use std::{error::Error, fs, io, path::Path};

#[derive(Clone, Copy)]
struct Case {
    phase: Phase,
    boundary: Boundary,
    previous: Phase,
    effect: Option<Effect>,
    uncertain: Option<Effect>,
    candidate_present: bool,
    gc: &'static [&'static str],
}

// Size: medium. Oracle for every law here: refusal is not rollback; exact typed
// boundary/effects/cause and actual namespace must agree, with no later work.
// EIO is injected at named I/O checkpoints; namespace changes use real ext4.
// Restart means drop and reopen, not physical power loss. No sleeps or random schedules.
// Delete only when stronger public failure/restart laws subsume these fault cells.
#[test]
fn candidate_unlink_failure_reports_uncertainty_before_reobservation() -> Result<(), Box<dyn Error>>
{
    require_failure(Case {
        phase: Phase::UnlinkCandidate,
        boundary: Boundary::CandidateUnlink,
        previous: Phase::SynchronizeGcAfterIntentCleanup,
        effect: None,
        uncertain: Some(Effect::CandidateRemoved),
        candidate_present: true,
        gc: &["intent"],
    })
}

// Size: medium. Oracle/deletion: failure/restart contract above.
#[test]
fn candidate_absence_failure_reports_the_completed_unlink() -> Result<(), Box<dyn Error>> {
    require_failure(Case {
        phase: Phase::UnlinkCandidate,
        boundary: Boundary::CandidateAbsence,
        previous: Phase::SynchronizeGcAfterIntentCleanup,
        effect: Some(Effect::CandidateRemoved),
        uncertain: None,
        candidate_present: false,
        gc: &["intent"],
    })
}

// Size: medium. Oracle/deletion: failure/restart contract above.
#[test]
fn intent_absence_failure_preserves_the_completed_receipt() -> Result<(), Box<dyn Error>> {
    require_failure(Case {
        phase: Phase::RemoveIntent,
        boundary: Boundary::IntentAbsence,
        previous: Phase::SynchronizeGcAfterReceipt,
        effect: Some(Effect::IntentRemoved),
        uncertain: None,
        candidate_present: false,
        gc: &["receipt"],
    })
}

// Size: medium. Oracle/deletion: failure/restart contract above.
#[test]
fn intent_sync_failure_does_not_remove_its_retained_stage() -> Result<(), Box<dyn Error>> {
    require_failure(Case {
        phase: Phase::SynchronizeGcAfterIntent,
        boundary: Boundary::GcIntentSynchronization,
        previous: Phase::LinkIntent,
        effect: None,
        uncertain: None,
        candidate_present: true,
        gc: &["intent", "intent.next"],
    })
}

// Size: medium. Oracle/deletion: failure/restart contract above.
#[test]
fn intent_cleanup_sync_failure_does_not_retire_any_candidate() -> Result<(), Box<dyn Error>> {
    require_failure(Case {
        phase: Phase::SynchronizeGcAfterIntentCleanup,
        boundary: Boundary::GcIntentCleanupSynchronization,
        previous: Phase::RemoveIntentStage,
        effect: None,
        uncertain: None,
        candidate_present: true,
        gc: &["intent"],
    })
}

// Size: medium. Oracle/deletion: failure/restart contract above.
#[test]
fn segment_sync_failure_does_not_undo_candidate_removal() -> Result<(), Box<dyn Error>> {
    require_failure(Case {
        phase: Phase::SynchronizeSegmentPool,
        boundary: Boundary::PoolSynchronization,
        previous: Phase::UnlinkCandidate,
        effect: None,
        uncertain: None,
        candidate_present: false,
        gc: &["intent"],
    })
}

// Size: medium. Oracle/deletion: failure/restart contract above.
#[test]
fn receipt_sync_failure_preserves_intent_for_recovery() -> Result<(), Box<dyn Error>> {
    require_failure(Case {
        phase: Phase::SynchronizeGcAfterReceipt,
        boundary: Boundary::GcReceiptSynchronization,
        previous: Phase::ReplaceReceipt,
        effect: None,
        uncertain: None,
        candidate_present: false,
        gc: &["intent", "receipt"],
    })
}

// Size: medium. Oracle/deletion: failure/restart contract above.
#[test]
fn intent_removal_sync_failure_retains_the_final_receipt() -> Result<(), Box<dyn Error>> {
    require_failure(Case {
        phase: Phase::SynchronizeGcAfterIntentRemoval,
        boundary: Boundary::GcIntentRemovalSynchronization,
        previous: Phase::RemoveIntent,
        effect: None,
        uncertain: None,
        candidate_present: false,
        gc: &["receipt"],
    })
}

fn require_failure(case: Case) -> Result<(), Box<dyn Error>> {
    let sandbox = disposed_store(&format!("gc-effect-{:?}", case.boundary))?;
    let expected_live = fs::read(sandbox.path().join("segments").join(LIVE_SEGMENT_NAME))?;
    let expected_orphan = fs::read(sandbox.path().join("segments").join(ORPHAN_SEGMENT_NAME))?;
    let plan = plan(sandbox.path())?;
    let mut writer = authority(sandbox.path())?;
    let _prepared = writer.prepare(&plan)?;
    let intent = writer
        .bound_intent()
        .ok_or("intent absent")?
        .encoded()
        .to_vec();
    writer.storage_failure = Some(case.boundary);
    let Err(FilesystemGcError::Execute(error)) = writer.execute(&plan) else {
        return Err("injected storage error was not returned as execution failure".into());
    };
    assert_report(&error, &case)?;
    assert!(
        writer.bound_intent().is_none(),
        "failed public execution must invalidate its context"
    );
    assert_eq!(
        gc_entries(sandbox.path())?,
        case.gc,
        "no later phase may run"
    );
    assert_candidate(sandbox.path(), &case, &expected_orphan)?;
    assert_eq!(
        fs::read(sandbox.path().join("segments").join(LIVE_SEGMENT_NAME))?,
        expected_live,
        "live bytes survive failure"
    );
    if case.gc.contains(&"intent") {
        assert_eq!(
            fs::read(sandbox.path().join("gc/intent"))?,
            intent,
            "original recovery authority survives"
        );
    }
    let retained_receipt = if case.gc.contains(&"receipt") {
        Some(fs::read(sandbox.path().join("gc/receipt"))?)
    } else {
        None
    };
    drop(writer);
    assert_restart(sandbox.path(), &expected_live, retained_receipt)
}

fn assert_restart(
    root: &Path,
    expected_live: &[u8],
    retained_receipt: Option<Vec<u8>>,
) -> Result<(), Box<dyn Error>> {
    let report = authority(root)?.recover()?;
    assert!(
        report.receipt().is_some(),
        "fresh recovery must complete the retirement"
    );
    assert_eq!(gc_entries(root)?, ["receipt"]);
    assert_eq!(
        fs::read(root.join("segments").join(LIVE_SEGMENT_NAME))?,
        expected_live
    );
    assert_eq!(
        fs::metadata(root.join("segments").join(ORPHAN_SEGMENT_NAME))
            .err()
            .map(|e| e.kind()),
        Some(io::ErrorKind::NotFound)
    );
    if let Some(bytes) = retained_receipt {
        assert_eq!(
            fs::read(root.join("gc/receipt"))?,
            bytes,
            "recovery preserves completed receipt bytes"
        );
    }
    assert_eq!(authority(root)?.recover()?.plan(), GcRecoveryPlan::Complete);
    Ok(())
}

fn assert_report(error: &GcExecutionError, case: &Case) -> Result<(), Box<dyn Error>> {
    assert_eq!(
        error.point(),
        GcExecutionPoint::at(case.phase),
        "failed public phase"
    );
    assert_eq!(
        error.executed().last(),
        Some(&GcExecutionPoint::at(case.previous)),
        "last successful call; not a durability claim"
    );
    let progress = error
        .storage_progress()
        .ok_or("failed capability effects are unreported")?;
    assert_eq!(progress.boundary(), case.boundary);
    let known: Vec<_> = progress
        .known_effects()
        .iter()
        .map(|e| (e.effect(), e.durability()))
        .collect();
    let expected: Vec<_> = case
        .effect
        .into_iter()
        .map(|e| (e, Durability::Unconfirmed))
        .collect();
    assert_eq!(
        known, expected,
        "failing capability known effects and durability"
    );
    assert_eq!(
        progress.uncertain_effect(),
        case.uncertain,
        "uncertain is distinct from known or absent"
    );
    assert_eq!(os_cause(error), Some(5), "original injected OS cause");
    Ok(())
}

fn assert_candidate(root: &Path, case: &Case, expected: &[u8]) -> Result<(), Box<dyn Error>> {
    let path = root.join("segments").join(ORPHAN_SEGMENT_NAME);
    if case.candidate_present {
        assert_eq!(
            fs::read(path)?,
            expected,
            "candidate remains exact before removal"
        );
    } else {
        assert_eq!(
            fs::metadata(path).err().map(|e| e.kind()),
            Some(io::ErrorKind::NotFound),
            "successful unlink is not rolled back"
        );
    }
    Ok(())
}

fn os_cause(mut error: &(dyn Error + 'static)) -> Option<i32> {
    loop {
        if let Some(io) = error.downcast_ref::<io::Error>() {
            if let Some(code) = io.raw_os_error() {
                return Some(code);
            }
            if let Some(inner) = io.get_ref() {
                error = inner;
                continue;
            }
        }
        error = error.source()?;
    }
}

// Size: medium. Oracle: a changed candidate refuses before unlink and retains
// the exact contradiction; this is distinct from uncertainty after an attempted syscall.
// Delete only if stronger admission/error-source laws subsume this outcome.
#[test]
fn candidate_verification_refusal_reports_no_removal() -> Result<(), Box<dyn Error>> {
    let sandbox = disposed_store("gc-effect-candidate-refusal")?;
    let plan = plan(sandbox.path())?;
    let mut writer = authority(sandbox.path())?;
    let prepared = writer.prepare(&plan)?;
    let path = sandbox.path().join("segments").join(ORPHAN_SEGMENT_NAME);
    fs::write(&path, b"contradiction")?;
    let Err(error) = crate::adapters::gc::execute_gc(&mut writer, prepared.candidate_count())
    else {
        return Err("changed candidate was retired".into());
    };
    let progress = error
        .storage_progress()
        .ok_or("pre-effect refusal is unreported")?;
    assert_eq!(error.point(), GcExecutionPoint::at(Phase::UnlinkCandidate));
    assert_eq!(progress.boundary(), Boundary::CandidateVerification);
    assert!(
        matches!(
            gc_refusal(&error),
            Some(super::super::FilesystemGcRefusal::CandidateKindOrLength)
        ),
        "original candidate refusal"
    );
    assert!(
        progress.known_effects().is_empty(),
        "verification must not initiate removal"
    );
    assert_eq!(progress.uncertain_effect(), None);
    assert_eq!(
        fs::read(path)?,
        b"contradiction",
        "refusal preserves observed evidence"
    );
    assert_eq!(
        gc_entries(sandbox.path())?,
        ["intent"],
        "receipt publication must not run"
    );
    Ok(())
}

fn gc_refusal<'a>(
    mut error: &'a (dyn Error + 'static),
) -> Option<&'a super::super::FilesystemGcRefusal> {
    loop {
        if let Some(refusal) = error.downcast_ref::<super::super::FilesystemGcRefusal>() {
            return Some(refusal);
        }
        error = match error
            .downcast_ref::<io::Error>()
            .and_then(io::Error::get_ref)
        {
            Some(inner) => inner,
            None => error.source()?,
        };
    }
}
