//! This module owns post-process-death verification of retention publication.
//!
//! After the child dies at its coordinate, restart reopens the migrated store
//! through the same admission a production caller would use, runs retention
//! recovery, and requires the documented steps and outcome for that exact
//! prefix; then it requires the forward retry to report the outcome recovery
//! predicts.

use std::io;
use std::path::Path;

use keep::{
    RetentionCurrentStateRefusal, RetentionPublicationError, RetentionPublicationOutcome,
    RetentionRecoveryOutcome as Outcome, RetentionRecoveryStep as Step,
    execute_retention_publication,
};
use xtask::{DurabilityCrashCase, DurabilityCrashPoint, DurabilityCrashPosition};

use super::super::DurabilityCrashMatrixError;
use super::super::production_protocol::fixture::GoldenFixture;
use super::super::production_protocol::retention::{preparation, reopened_authority};
use super::super::production_protocol::verification;

const PROTECTED_ROOT: Outcome = Outcome::Protected {
    root_stage: true,
    manifest_stage: false,
};
const PROTECTED_BOTH: Outcome = Outcome::Protected {
    root_stage: true,
    manifest_stage: true,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Retry {
    Published,
    AlreadyCommitted,
    Refused,
}

pub(super) fn verify(
    store_root: &Path,
    case: DurabilityCrashCase,
) -> Result<(), DurabilityCrashMatrixError> {
    let (steps, outcome, retry) = expected(case);
    let mut authority = reopened_authority(store_root)?;
    let receipt = authority
        .recover()
        .map_err(|source| verification("recover crash retention stages", source))?;
    if receipt.executed() != steps.as_slice() || receipt.outcome() != outcome {
        return Err(mismatch(format!(
            "{} {:?}: expected {steps:?} -> {outcome:?}, recovered {:?} -> {:?}",
            case.point().identifier(),
            case.position(),
            receipt.executed(),
            receipt.outcome()
        )));
    }
    let root = GoldenFixture::retention_root()?;
    let preparation = preparation(root.bytes())?;
    match (
        retry,
        execute_retention_publication(&mut authority, &preparation),
    ) {
        (Retry::Published, Ok(receipt))
            if receipt.outcome() == RetentionPublicationOutcome::Published => {}
        (Retry::AlreadyCommitted, Ok(receipt))
            if receipt.outcome() == RetentionPublicationOutcome::AlreadyCommitted => {}
        (Retry::Refused, Err(RetentionPublicationError::CurrentVerification { source }))
            if source
                .get_ref()
                .and_then(|refusal| refusal.downcast_ref::<RetentionCurrentStateRefusal>())
                .is_some_and(|refusal| {
                    matches!(refusal, RetentionCurrentStateRefusal::RetainedStage)
                }) => {}
        (retry, result) => {
            return Err(mismatch(format!(
                "{} {:?}: expected forward retry {retry:?}, got {result:?}",
                case.point().identifier(),
                case.position()
            )));
        }
    }
    Ok(())
}

fn mismatch(message: String) -> DurabilityCrashMatrixError {
    verification("verify crash retention recovery", io::Error::other(message))
}

/// The number of completed publication phases and any truncated stage the
/// coordinate leaves behind.
fn prefix(case: DurabilityCrashCase) -> (usize, Option<Step>) {
    let index = DurabilityCrashPoint::ALL
        .iter()
        .position(|point| *point == case.point())
        .and_then(|index| index.checked_sub(35))
        .unwrap_or(0);
    // Phase 1 is current-state verification; point n is phase n + 2.
    let phase = index.saturating_add(2);
    let write = match case.point() {
        DurabilityCrashPoint::WriteRootStage => Some(Step::DiscardRootStage),
        DurabilityCrashPoint::WriteManifestStage => Some(Step::DiscardManifestStage),
        DurabilityCrashPoint::WriteHeadStage => Some(Step::DiscardHeadStage),
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
        DurabilityCrashPoint::AdmitRootNamespace
            | DurabilityCrashPoint::LinkRoot
            | DurabilityCrashPoint::LinkManifest
            | DurabilityCrashPoint::ReplaceRetentionHead
            | DurabilityCrashPoint::RemoveRootStage
            | DurabilityCrashPoint::RemoveManifestStage
    )
}

/// The documented recovery for the prefix a coordinate leaves behind.
fn expected(case: DurabilityCrashCase) -> (Vec<Step>, Outcome, Retry) {
    let (count, truncated) = prefix(case);
    let (mut steps, outcome, retry) = match count {
        0 | 1 => (vec![], Outcome::Clean, Retry::Published),
        2..=5 => (vec![Step::LinkRoot], PROTECTED_ROOT, Retry::Refused),
        6 | 7 => (vec![], PROTECTED_ROOT, Retry::Refused),
        8 | 9 => (vec![Step::LinkManifest], PROTECTED_BOTH, Retry::Refused),
        10 | 11 => (vec![], PROTECTED_BOTH, Retry::Refused),
        12 | 13 => (
            vec![
                Step::FinalizeHead,
                Step::RemoveRootStage,
                Step::RemoveManifestStage,
            ],
            Outcome::Committed,
            Retry::AlreadyCommitted,
        ),
        14 | 15 => (
            vec![Step::RemoveRootStage, Step::RemoveManifestStage],
            Outcome::Committed,
            Retry::AlreadyCommitted,
        ),
        16 => (
            vec![Step::RemoveManifestStage],
            Outcome::Committed,
            Retry::AlreadyCommitted,
        ),
        _ => (vec![], Outcome::Clean, Retry::AlreadyCommitted),
    };
    if let Some(discard) = truncated {
        steps.insert(0, discard);
    }
    (steps, outcome, retry)
}
