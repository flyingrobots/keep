//! This module owns pure planning of one explicit finalize-or-retire
//! disposition over a recovery-protected retention orphan.

use std::fmt;

use super::{
    RetentionPoolEntryObservation as Pool, RetentionPoolObservations, RetentionRecoveryOutcome,
    RetentionRecoveryPlan,
};
use crate::adapters::RecoveryDispositionDecision;

/// Which recovery-protected stage one disposition names.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecoveryDispositionTarget {
    /// The complete, linked, retained `retention/root.next`.
    Root,
    /// The complete, linked, retained `retention/manifest.next`.
    Manifest,
}

/// One explicit decision over one protected stage.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RecoveryDispositionRequest {
    /// The protected stage.
    pub target: RecoveryDispositionTarget,
    /// The decision: `Finalize` keeps the linked pool entry as a durable
    /// immutable artifact a byte-identical publication may reuse; `Retire`
    /// additionally marks it collectible by a future retention-pool
    /// collector. Both remove the retained stage so publication may proceed.
    pub decision: RecoveryDispositionDecision,
}

/// The ordered durable phases one disposition executes.
///
/// The receipt is durable before the retained stage is removed, so process
/// death anywhere leaves either a recoverable stage or a durable decision. A
/// `Retire` decision additionally unlinks the immutable pool entry, because
/// an absent retention head admits no pool artifact and no collector exists
/// for retention pools; a `Finalize` decision ends at the retention
/// synchronization and keeps the entry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecoveryDispositionPhase {
    /// Write the complete canonical receipt to `recovery/disposition.next`.
    WriteStage,
    /// Synchronize `recovery/disposition.next`.
    SynchronizeStage,
    /// Link the stage to `recovery/dispositions/<artifact-digest>.receipt`
    /// without replacement.
    LinkReceipt,
    /// Synchronize `recovery/dispositions`.
    SynchronizeDispositions,
    /// Remove the retained `recovery/disposition.next`.
    RemoveStage,
    /// Synchronize `recovery`.
    SynchronizeRecovery,
    /// Remove the retained retention stage after proving its pool link.
    RemoveRetainedStage,
    /// Synchronize `retention`.
    SynchronizeRetention,
    /// Unlink the retired artifact from its immutable pool after proving
    /// its exact bytes (`Retire` only).
    RemovePoolEntry,
    /// Synchronize the pool the artifact left (`Retire` only).
    SynchronizePool,
}

impl RecoveryDispositionPhase {
    /// Every phase in execution order.
    pub const ALL: [Self; 10] = [
        Self::WriteStage,
        Self::SynchronizeStage,
        Self::LinkReceipt,
        Self::SynchronizeDispositions,
        Self::RemoveStage,
        Self::SynchronizeRecovery,
        Self::RemoveRetainedStage,
        Self::SynchronizeRetention,
        Self::RemovePoolEntry,
        Self::SynchronizePool,
    ];

    /// The phases one decision executes, in order.
    #[must_use]
    pub fn for_decision(decision: RecoveryDispositionDecision) -> &'static [Self] {
        match decision {
            RecoveryDispositionDecision::Finalize => Self::ALL.get(..8).unwrap_or(&[]),
            RecoveryDispositionDecision::Retire => &Self::ALL,
        }
    }
}

/// The one lawful disposition of a request over observed evidence.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RecoveryDispositionPlan {
    target: RecoveryDispositionTarget,
    decision: RecoveryDispositionDecision,
}

impl RecoveryDispositionPlan {
    /// Returns the protected stage the plan disposes.
    pub const fn target(self) -> RecoveryDispositionTarget {
        self.target
    }

    /// Returns the decision.
    pub const fn decision(self) -> RecoveryDispositionDecision {
        self.decision
    }
}

/// Why a request cannot be planned over the observed evidence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecoveryDispositionRefusal {
    /// Recovery still has steps to execute; run it first.
    RecoveryPending,
    /// No stage is recovery-protected; there is nothing to dispose.
    NothingProtected,
    /// The requested stage is not retained.
    TargetNotRetained {
        /// The requested stage.
        target: RecoveryDispositionTarget,
    },
    /// The root stage cannot be disposed while the manifest stage that names
    /// it is still protected; dispose the manifest first.
    ManifestStageRemains,
    /// The requested stage's pool entry is not the linked, identical entry
    /// recovery proved.
    TargetNotLinked {
        /// The requested stage.
        target: RecoveryDispositionTarget,
        /// What the pool holds.
        observed: Pool,
    },
    /// No retention head is published, so there is no visible state to
    /// finalize the artifact into; only `Retire` applies.
    FinalizeRequiresPublishedHead,
}

impl fmt::Display for RecoveryDispositionRefusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RecoveryPending => {
                formatter.write_str("retention recovery has steps to run before any disposition")
            }
            Self::NothingProtected => formatter.write_str("no retention stage is protected"),
            Self::TargetNotRetained { target } => {
                write!(formatter, "the {target:?} stage is not retained")
            }
            Self::ManifestStageRemains => formatter
                .write_str("the manifest stage names the root stage; dispose the manifest first"),
            Self::TargetNotLinked { target, observed } => write!(
                formatter,
                "the {target:?} stage's pool entry is {observed:?}, not identical"
            ),
            Self::FinalizeRequiresPublishedHead => {
                formatter.write_str("no retention head is published; an orphan can only be retired")
            }
        }
    }
}

impl std::error::Error for RecoveryDispositionRefusal {}

/// Plans one disposition over the recovery plan and pool observations
/// restart established.
///
/// The recovery plan must already be fully executed (no pending steps) and
/// must protect the requested stage; the stage's pool entry must be the
/// identical linked entry; `Finalize` needs a published retention head.
///
/// # Errors
///
/// Returns [`RecoveryDispositionRefusal`] naming the exact reason.
pub fn plan_recovery_disposition(
    recovery: &RetentionRecoveryPlan,
    pools: RetentionPoolObservations,
    head_published: bool,
    request: RecoveryDispositionRequest,
) -> Result<RecoveryDispositionPlan, RecoveryDispositionRefusal> {
    if !recovery.steps().is_empty() {
        return Err(RecoveryDispositionRefusal::RecoveryPending);
    }
    if request.decision == RecoveryDispositionDecision::Finalize && !head_published {
        return Err(RecoveryDispositionRefusal::FinalizeRequiresPublishedHead);
    }
    let RetentionRecoveryOutcome::Protected {
        root_stage,
        manifest_stage,
    } = recovery.outcome()
    else {
        return Err(RecoveryDispositionRefusal::NothingProtected);
    };
    let (retained, observed) = match request.target {
        RecoveryDispositionTarget::Root => (root_stage, pools.root),
        RecoveryDispositionTarget::Manifest => (manifest_stage, pools.manifest),
    };
    if !retained {
        return Err(RecoveryDispositionRefusal::TargetNotRetained {
            target: request.target,
        });
    }
    if request.target == RecoveryDispositionTarget::Root && manifest_stage {
        return Err(RecoveryDispositionRefusal::ManifestStageRemains);
    }
    if observed != Pool::Identical {
        return Err(RecoveryDispositionRefusal::TargetNotLinked {
            target: request.target,
            observed,
        });
    }
    Ok(RecoveryDispositionPlan {
        target: request.target,
        decision: request.decision,
    })
}
