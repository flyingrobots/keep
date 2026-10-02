//! This module owns ordered execution of one retention recovery plan.

use super::RetentionStorageError;
use std::error::Error;
use std::fmt;

use super::{
    RetentionRecoveryOutcome, RetentionRecoveryPlan, RetentionRecoveryStep,
    RetentionRecoveryStorage,
};

/// The complete record of one executed retention recovery plan.
#[derive(Clone, Debug, Eq, PartialEq)]
#[must_use]
pub struct RetentionRecoveryReceipt {
    executed: Vec<RetentionRecoveryStep>,
    outcome: RetentionRecoveryOutcome,
}

impl RetentionRecoveryReceipt {
    /// Every step that executed, in order.
    #[must_use]
    pub fn executed(&self) -> &[RetentionRecoveryStep] {
        &self.executed
    }

    /// The state the store is in now.
    #[must_use]
    pub const fn outcome(&self) -> RetentionRecoveryOutcome {
        self.outcome
    }
}

/// One failed recovery step and the steps that completed before it.
#[derive(Debug)]
pub struct RetentionRecoveryError {
    step: RetentionRecoveryStep,
    executed: Vec<RetentionRecoveryStep>,
    source: RetentionStorageError,
}

impl RetentionRecoveryError {
    /// Effects reported by the failing capability, independently of preceding completed steps.
    ///
    /// `None` means that adapter did not report its effects; it does not mean no mutation.
    #[must_use]
    pub const fn progress(&self) -> Option<&super::RetentionStorageProgress> {
        self.source.progress()
    }
    /// The precise storage failure, without dynamic downcasting.
    #[must_use]
    pub const fn storage_error(&self) -> &RetentionStorageError {
        &self.source
    }

    /// The step that failed, possibly after effects.
    #[must_use]
    pub const fn step(&self) -> RetentionRecoveryStep {
        self.step
    }

    /// Every step that completed before the failure, in order. This excludes effects of the failing step.
    #[must_use]
    pub fn executed(&self) -> &[RetentionRecoveryStep] {
        &self.executed
    }
}

impl fmt::Display for RetentionRecoveryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "retention recovery step {:?} failed after {} completed step(s)",
            self.step,
            self.executed.len()
        )
    }
}

impl Error for RetentionRecoveryError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.source)
    }
}

/// Executes `plan` against `storage` in order, stopping at the first error.
///
/// Each step calls exactly one storage capability. A failed step leaves the
/// completed steps' effects in place, names the step, and returns; the caller
/// re-observes and re-plans rather than continuing from stale evidence. The
/// failing capability may also have effects: inspect its reported progress.
/// Missing progress is uncertainty, never a claim that nothing changed.
///
/// # Errors
///
/// Returns [`RetentionRecoveryError`] with the failed step, the completed
/// steps, and the storage's own error as source.
pub fn execute_retention_recovery<S: RetentionRecoveryStorage>(
    storage: &mut S,
    plan: &RetentionRecoveryPlan,
) -> Result<RetentionRecoveryReceipt, RetentionRecoveryError> {
    let mut executed = Vec::with_capacity(plan.steps().len());
    for &step in plan.steps() {
        let result = match step {
            RetentionRecoveryStep::DiscardHeadStage => storage.discard_head_stage(),
            RetentionRecoveryStep::DiscardManifestStage => storage.discard_manifest_stage(),
            RetentionRecoveryStep::DiscardRootStage => storage.discard_root_stage(),
            RetentionRecoveryStep::LinkRoot => storage.link_root(),
            RetentionRecoveryStep::LinkManifest => storage.link_manifest(),
            RetentionRecoveryStep::FinalizeHead => storage.finalize_head(),
            RetentionRecoveryStep::RemoveRootStage => storage.remove_root_stage(),
            RetentionRecoveryStep::RemoveManifestStage => storage.remove_manifest_stage(),
        };
        if let Err(source) = result {
            return Err(RetentionRecoveryError {
                step,
                executed,
                source,
            });
        }
        executed.push(step);
    }
    Ok(RetentionRecoveryReceipt {
        executed,
        outcome: plan.outcome(),
    })
}
