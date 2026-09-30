//! This module owns independent post-process-death GC verification: the
//! live segment is never lost, the residue plans exactly the documented
//! recovery, the production recovery reaches one complete retirement, and
//! a fresh plan then has nothing to retire.

use std::fs;
use std::io;
use std::path::Path;

use keep::{GcExecutionPhase, GcExecutionPoint, GcFixedStage, GcRecoveryPlan};
use xtask::{DurabilityCrashCase, DurabilityCrashPoint, DurabilityCrashPosition};

use super::super::DurabilityCrashMatrixError;
use super::super::production_protocol::fixture::{
    BUNDLE_SEGMENT_NAME, GoldenFixture, SEGMENT_POOL_PATH,
};
use super::super::production_protocol::gc::{gc_authority, plan};
use super::super::production_protocol::verification;

pub(super) fn verify(
    store_root: &Path,
    case: DurabilityCrashCase,
) -> Result<(), DurabilityCrashMatrixError> {
    require_live_segment(store_root)?;
    let (count, truncated) = prefix(case);
    if let Some(stage) = truncated {
        let report = recover(store_root)?;
        if report != (GcRecoveryPlan::DiscardStage { stage }) {
            return Err(mismatch(
                case,
                format!("expected discard of {stage:?}, got {report:?}"),
            ));
        }
    }
    let report = recover(store_root)?;
    let expected = expected_plan(count);
    if report != expected {
        return Err(mismatch(
            case,
            format!("expected {expected:?}, recovered {report:?}"),
        ));
    }
    if report == GcRecoveryPlan::Idle {
        let plan = plan(store_root)?;
        let _receipt = gc_authority(store_root)?
            .execute(&plan)
            .map_err(|source| verification("execute forward GC retirement", source))?;
    }
    require_complete(store_root, case)
}

fn require_live_segment(store_root: &Path) -> Result<(), DurabilityCrashMatrixError> {
    let observed = fs::read(store_root.join("segments").join(BUNDLE_SEGMENT_NAME))
        .map_err(|source| DurabilityCrashMatrixError::io("read live crash segment", source))?;
    let expected = GoldenFixture::bundle_segment()?;
    if observed == expected.bytes() {
        Ok(())
    } else {
        Err(DurabilityCrashMatrixError::artifact_bytes(
            "segments/<bundle segment>",
            expected.bytes(),
            &observed,
        ))
    }
}

fn require_complete(
    store_root: &Path,
    case: DurabilityCrashCase,
) -> Result<(), DurabilityCrashMatrixError> {
    require_live_segment(store_root)?;
    if store_root.join(SEGMENT_POOL_PATH).exists() {
        return Err(mismatch(case, "the retired orphan is still present".into()));
    }
    let mut entries: Vec<String> = fs::read_dir(store_root.join("gc"))
        .map_err(|source| DurabilityCrashMatrixError::io("list crash gc directory", source))?
        .map(|entry| entry.map(|entry| entry.file_name().to_string_lossy().into_owned()))
        .collect::<Result<_, io::Error>>()
        .map_err(|source| DurabilityCrashMatrixError::io("read crash gc entry", source))?;
    entries.sort();
    if entries != ["receipt"] {
        return Err(mismatch(
            case,
            format!("gc holds {entries:?}, expected only the receipt"),
        ));
    }
    let plan = plan(store_root)?;
    if plan.candidate_count() != 0 || plan.already_retired().len() != 1 {
        return Err(mismatch(case, "a fresh plan still names the orphan".into()));
    }
    let settled = recover(store_root)?;
    if settled == GcRecoveryPlan::Complete {
        Ok(())
    } else {
        Err(mismatch(
            case,
            format!("settled residue planned {settled:?}"),
        ))
    }
}

/// Reacquires writer authority the way a restarted process would and runs
/// the production recovery once, reporting the plan the residue admitted.
fn recover(store_root: &Path) -> Result<GcRecoveryPlan, DurabilityCrashMatrixError> {
    gc_authority(store_root)?
        .recover()
        .map(|report| report.plan())
        .map_err(|source| verification("recover production GC retirement", source))
}

fn mismatch(case: DurabilityCrashCase, message: String) -> DurabilityCrashMatrixError {
    verification(
        "verify crash GC recovery",
        io::Error::other(format!(
            "{} {:?}: {message}",
            case.point().identifier(),
            case.position()
        )),
    )
}

/// The number of completed points and any truncated stage the coordinate
/// leaves behind, for the one-candidate retirement.
fn prefix(case: DurabilityCrashCase) -> (usize, Option<GcFixedStage>) {
    let phase = DurabilityCrashPoint::GC
        .iter()
        .position(|point| *point == case.point())
        .map_or(0, |index| index.saturating_add(1));
    let write = match case.point() {
        DurabilityCrashPoint::GcWriteIntentStage => Some(GcFixedStage::Intent),
        DurabilityCrashPoint::GcWriteReceiptStage => Some(GcFixedStage::Receipt),
        _ => None,
    };
    match case.position() {
        DurabilityCrashPosition::After => (phase, None),
        DurabilityCrashPosition::During if write.is_some() => (phase.saturating_sub(1), write),
        DurabilityCrashPosition::During if atomic(case.point()) => (phase, None),
        DurabilityCrashPosition::Before | DurabilityCrashPosition::During => {
            (phase.saturating_sub(1), None)
        }
    }
}

const fn atomic(point: DurabilityCrashPoint) -> bool {
    matches!(
        point,
        DurabilityCrashPoint::GcLinkIntent
            | DurabilityCrashPoint::GcRemoveIntentStage
            | DurabilityCrashPoint::GcUnlinkCandidate
            | DurabilityCrashPoint::GcReplaceReceipt
            | DurabilityCrashPoint::GcRemoveIntent
    )
}

/// The documented recovery for a process death after `count` points, the
/// state table in `gc.md`.
fn expected_plan(count: usize) -> GcRecoveryPlan {
    let resume = |phase| GcRecoveryPlan::Resume {
        from: GcExecutionPoint::at(phase),
    };
    match count {
        0 => GcRecoveryPlan::Idle,
        1 | 2 => resume(GcExecutionPhase::SynchronizeIntentStage),
        3 | 4 => resume(GcExecutionPhase::SynchronizeGcAfterIntent),
        5 | 6 => resume(GcExecutionPhase::SynchronizeGcAfterIntentCleanup),
        7 | 8 => resume(GcExecutionPhase::WriteReceiptStage),
        9 | 10 => resume(GcExecutionPhase::SynchronizeReceiptStage),
        11 | 12 => resume(GcExecutionPhase::SynchronizeGcAfterReceipt),
        _ => GcRecoveryPlan::Complete,
    }
}
