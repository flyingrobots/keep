//! This module owns ordered execution of one retention recovery plan.

use std::error::Error;
use std::fmt;
use std::io;

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

/// One refused recovery step and the steps that completed before it.
#[derive(Debug)]
pub struct RetentionRecoveryError {
    step: RetentionRecoveryStep,
    executed: Vec<RetentionRecoveryStep>,
    source: io::Error,
}

impl RetentionRecoveryError {
    /// The step that refused.
    #[must_use]
    pub const fn step(&self) -> RetentionRecoveryStep {
        self.step
    }

    /// Every step that completed before the refusal, in order.
    #[must_use]
    pub fn executed(&self) -> &[RetentionRecoveryStep] {
        &self.executed
    }
}

impl fmt::Display for RetentionRecoveryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "retention recovery step {:?} refused after {} completed step(s)",
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

/// Executes `plan` against `storage` in order, stopping at the first refusal.
///
/// Each step calls exactly one storage capability. A refused step leaves the
/// completed steps' effects in place, names the step, and returns; the caller
/// re-observes and re-plans rather than continuing from stale evidence.
///
/// # Errors
///
/// Returns [`RetentionRecoveryError`] with the refused step, the completed
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
