//! This module owns the prepared recovery queue's ordered execution and effects.

use std::io;

use super::recovery::{CompactionRecovery, FilesystemCompactionRecoveryError as Error, refused};
use super::recovery_failure::{discard_failure, finalization_failure};
use super::recovery_observation::ObservedStage;
use super::recovery_preflight::{
    NextHeadAction, PreparedNextHead, PreparedStage, RecoveryPlan, StageAction,
};
use super::recovery_schedule::{Schedule, ScheduledDiscard, ScheduledFinalization};
use super::{
    CompactionRecoveryAction as Action, CompactionRecoveryBoundary as Boundary,
    CompactionRecoveryEffects as Effects,
};
use crate::adapters::filesystem_catalog_artifact::synchronize_directory;
use crate::adapters::filesystem_exact_record as exact_record;
use crate::adapters::{
    CatalogRestartPolicy, FilesystemRecoveryNextHeadFinalizer, FilesystemRecoveryStageDiscarder,
    RecoveryStage, RecoveryStageParent, execute_recovery_next_head_finalization,
    execute_recovery_stage_discard,
};

pub(super) fn execute(
    mut discarder: FilesystemRecoveryStageDiscarder,
    policy: CatalogRestartPolicy,
    plan: RecoveryPlan,
    schedule: &mut Schedule<'_>,
) -> Result<CompactionRecovery, Error> {
    let mut completed = CompactionRecovery {
        discarded: Vec::new(),
        finalized: None,
    };
    for stage in plan.stages {
        let name = stage.observed.stage();
        execute_stage(&mut discarder, stage, &completed, schedule)?;
        completed.discarded.push(name);
    }
    if let Some(next) = plan.next {
        execute_head(discarder, policy, next, &mut completed, schedule)?;
    }
    Ok(completed)
}

fn execute_stage(
    discarder: &mut FilesystemRecoveryStageDiscarder,
    mut stage: PreparedStage,
    completed: &CompactionRecovery,
    schedule: &mut Schedule<'_>,
) -> Result<(), Error> {
    let name = stage.observed.stage();
    stage.observed.verify(discarder).map_err(|error| {
        error.executing(
            completed,
            Action::Discard(name),
            Boundary::VerifyStage,
            Effects::None,
        )
    })?;
    match stage.action {
        StageAction::Derivable => {
            discard_derivable(discarder, &stage.observed, completed, schedule)
        }
        StageAction::Truncated(request) => {
            let _receipt = execute_recovery_stage_discard(
                &mut ScheduledDiscard {
                    inner: discarder,
                    stage: name,
                    schedule,
                },
                request,
            )
            .map_err(|source| discard_failure(source, completed, name))?;
            Ok(())
        }
    }
}

fn execute_head(
    mut discarder: FilesystemRecoveryStageDiscarder,
    policy: CatalogRestartPolicy,
    mut next: PreparedNextHead,
    completed: &mut CompactionRecovery,
    schedule: &mut Schedule<'_>,
) -> Result<(), Error> {
    let action = match next.action {
        NextHeadAction::Discard(_) => Action::Discard(RecoveryStage::NextHead),
        NextHeadAction::Finalize(request) => Action::Finalize(request.target().generation()),
    };
    next.observed.verify(&discarder).map_err(|error| {
        error.executing(completed, action, Boundary::VerifyStage, Effects::None)
    })?;
    match next.action {
        NextHeadAction::Discard(request) => {
            let _receipt = execute_recovery_stage_discard(
                &mut ScheduledDiscard {
                    inner: &mut discarder,
                    stage: RecoveryStage::NextHead,
                    schedule,
                },
                request,
            )
            .map_err(|source| discard_failure(source, completed, RecoveryStage::NextHead))?;
            completed.discarded.push(RecoveryStage::NextHead);
        }
        NextHeadAction::Finalize(request) => {
            let mut finalizer = ScheduledFinalization {
                inner: FilesystemRecoveryNextHeadFinalizer { discarder, policy },
                schedule,
            };
            let _receipt = execute_recovery_next_head_finalization(&mut finalizer, request)
                .map_err(|source| {
                    finalization_failure(source, completed, request.target().generation())
                })?;
            completed.finalized = Some(request.target().generation());
        }
    }
    Ok(())
}

/// The original opened evidence remains alive through unlink and synchronization.
fn discard_derivable(
    discarder: &FilesystemRecoveryStageDiscarder,
    observed: &ObservedStage,
    completed: &CompactionRecovery,
    schedule: &mut Schedule<'_>,
) -> Result<(), Error> {
    let stage = observed.stage();
    let action = Action::Discard(stage);
    let staging = discarder
        .inventory
        .parent_directory(RecoveryStageParent::Staging);
    schedule(stage, Boundary::RemoveStage).map_err(|source| {
        refused("before stage removal", source).executing(
            completed,
            action,
            Boundary::RemoveStage,
            Effects::None,
        )
    })?;
    staging.remove_file(stage.file_name()).map_err(|source| {
        refused("discard stage", source).executing(
            completed,
            action,
            Boundary::RemoveStage,
            Effects::Uncertain,
        )
    })?;
    schedule(stage, Boundary::ConfirmStageAbsent).map_err(|source| {
        refused("confirm stage absence", source).executing(
            completed,
            action,
            Boundary::ConfirmStageAbsent,
            Effects::AppliedUnconfirmed,
        )
    })?;
    exact_record::require_absent(staging, stage.file_name()).map_err(|source| {
        refused("confirm stage absence", source).executing(
            completed,
            action,
            Boundary::ConfirmStageAbsent,
            Effects::AppliedUnconfirmed,
        )
    })?;
    synchronize_removed_stage(staging, stage, completed, schedule)
}

fn synchronize_removed_stage(
    directory: &cap_std::fs::Dir,
    stage: RecoveryStage,
    completed: &CompactionRecovery,
    schedule: &mut Schedule<'_>,
) -> Result<(), Error> {
    let mut synchronize = || -> io::Result<()> {
        schedule(stage, Boundary::SynchronizeParent)?;
        synchronize_directory(directory)
    };
    synchronize().map_err(|source| {
        refused("synchronize staging", source).executing(
            completed,
            Action::Discard(stage),
            Boundary::SynchronizeParent,
            Effects::AppliedUnconfirmed,
        )
    })
}
