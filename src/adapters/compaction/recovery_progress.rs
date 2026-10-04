//! This module owns public execution progress for interrupted compaction recovery.

use super::CompactionRecovery;
use crate::{CatalogGeneration, RecoveryStage};

/// Recovery action that could not return a durable completion.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompactionRecoveryAction {
    /// Remove the exact observed stage and synchronize its parent.
    Discard(RecoveryStage),
    /// Replace the head with the exact candidate and synchronize the root.
    Finalize(CatalogGeneration),
}

/// Boundary at which the current recovery action stopped.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompactionRecoveryBoundary {
    /// Revalidate the originally observed stage before execution.
    VerifyStage,
    /// Remove the exact stage; a failed capability may have effects.
    RemoveStage,
    /// Confirm the stage name is absent after successful removal.
    ConfirmStageAbsent,
    /// Synchronize the removed stage's parent directory.
    SynchronizeParent,
    /// Revalidate current head and candidate before replacement.
    VerifyHead,
    /// Synchronize the candidate file before replacement.
    SynchronizeCandidate,
    /// Replace the head; a failed capability may have effects.
    ReplaceHead,
    /// Synchronize the root after replacement or already-current admission.
    SynchronizeRoot,
}

/// Namespace effects of the failing action alone; earlier actions are separate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompactionRecoveryEffects {
    /// This action made no namespace mutation. This is not a durability receipt.
    None,
    /// The action's namespace mutation succeeded, but directory durability was
    /// not confirmed by this action. Failure does not roll it back.
    AppliedUnconfirmed,
    /// The failing mutation capability does not establish whether its namespace
    /// mutation occurred or became durable. Fresh observation is required.
    Uncertain,
}

/// Progress when a prepared recovery action fails. No later action is executed.
///
/// Completed actions have synchronized their directories. The failed action's
/// effects are reported separately, so an empty completion list never proves
/// that the invocation made no changes. Retry must freshly observe the store.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompactionRecoveryExecution {
    pub(super) completed: CompactionRecovery,
    pub(super) action: CompactionRecoveryAction,
    pub(super) boundary: CompactionRecoveryBoundary,
    pub(super) effects: CompactionRecoveryEffects,
}

impl CompactionRecoveryExecution {
    /// Actions completed with successful directory synchronization before failure.
    #[must_use]
    pub const fn completed(&self) -> &CompactionRecovery {
        &self.completed
    }

    /// The action that stopped execution.
    #[must_use]
    pub const fn action(&self) -> CompactionRecoveryAction {
        self.action
    }

    /// The failed boundary within that action; the error retains its typed cause.
    #[must_use]
    pub const fn boundary(&self) -> CompactionRecoveryBoundary {
        self.boundary
    }

    /// Known or uncertain namespace effects of the failed action.
    #[must_use]
    pub const fn effects(&self) -> CompactionRecoveryEffects {
        self.effects
    }
}
