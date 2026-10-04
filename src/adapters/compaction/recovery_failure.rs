//! This module owns typed causes and execution evidence for recovery failures.

use std::{error::Error, fmt};

use super::{
    CompactionRecovery, CompactionRecoveryAction as Action, CompactionRecoveryBoundary as Boundary,
    CompactionRecoveryEffects as Effects, CompactionRecoveryExecution,
};
use crate::adapters::{
    RecoveryNextHeadFinalizationError as HeadError, RecoveryNextHeadFinalizationOutcome,
    RecoveryStage, RecoveryStageDiscardError as DiscardError, RecoveryStageDiscardOutcome,
    RecoveryStageDiscardStorageError,
};

/// Why recovery stopped. Admission refusal precedes all recovery mutations;
/// execution failure retains prior completions and failing-action effects.
#[derive(Debug)]
pub struct FilesystemCompactionRecoveryError {
    phase: &'static str,
    source: Box<dyn Error + Send + Sync>,
    execution: Option<Box<CompactionRecoveryExecution>>,
}

impl FilesystemCompactionRecoveryError {
    /// Progress for a failing prepared action, or `None` for admission refusal.
    /// Admission refusal initiates no recovery mutation; execution is not rollback.
    #[must_use]
    pub fn execution(&self) -> Option<&CompactionRecoveryExecution> {
        self.execution.as_deref()
    }

    /// Diagnostic phase; typed execution boundaries are available in `execution()`.
    #[must_use]
    pub const fn phase(&self) -> &'static str {
        self.phase
    }

    pub(in crate::adapters) fn refused(
        phase: &'static str,
        source: impl Error + Send + Sync + 'static,
    ) -> Self {
        Self {
            phase,
            source: Box::new(source),
            execution: None,
        }
    }

    pub(super) fn executing(
        mut self,
        completed: &CompactionRecovery,
        action: Action,
        boundary: Boundary,
        effects: Effects,
    ) -> Self {
        self.execution = Some(Box::new(CompactionRecoveryExecution {
            completed: completed.clone(),
            action,
            boundary,
            effects,
        }));
        self
    }
}

impl fmt::Display for FilesystemCompactionRecoveryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "compaction recovery failed at {}", self.phase)?;
        if let Some(progress) = self.execution() {
            write!(
                formatter,
                " (completed {:?}; action {:?}; boundary {:?}; effects {:?})",
                progress.completed(),
                progress.action(),
                progress.boundary(),
                progress.effects()
            )?;
        }
        Ok(())
    }
}

impl Error for FilesystemCompactionRecoveryError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.source.as_ref())
    }
}

pub(super) fn discard_failure(
    source: DiscardError,
    completed: &CompactionRecovery,
    stage: RecoveryStage,
) -> FilesystemCompactionRecoveryError {
    let (boundary, effects) = match &source {
        DiscardError::Remove {
            source: RecoveryStageDiscardStorageError::EvidenceMismatch { .. },
        } => (Boundary::RemoveStage, Effects::None),
        DiscardError::Remove { .. } => (Boundary::RemoveStage, Effects::Uncertain),
        DiscardError::Synchronize { outcome, .. } => (
            Boundary::SynchronizeParent,
            match outcome {
                RecoveryStageDiscardOutcome::Removed => Effects::AppliedUnconfirmed,
                RecoveryStageDiscardOutcome::AlreadyAbsent => Effects::None,
            },
        ),
    };
    FilesystemCompactionRecoveryError::refused("discard stage", source).executing(
        completed,
        Action::Discard(stage),
        boundary,
        effects,
    )
}

pub(super) fn finalization_failure(
    source: HeadError,
    completed: &CompactionRecovery,
    generation: crate::CatalogGeneration,
) -> FilesystemCompactionRecoveryError {
    let (boundary, effects) = match &source {
        HeadError::Verify { .. } => (Boundary::VerifyHead, Effects::None),
        HeadError::SynchronizeCandidate { .. } => (Boundary::SynchronizeCandidate, Effects::None),
        HeadError::Replace { .. } => (Boundary::ReplaceHead, Effects::Uncertain),
        HeadError::SynchronizeRoot { outcome, .. } => (
            Boundary::SynchronizeRoot,
            match outcome {
                RecoveryNextHeadFinalizationOutcome::Finalized => Effects::AppliedUnconfirmed,
                RecoveryNextHeadFinalizationOutcome::AlreadyFinalized => Effects::None,
            },
        ),
    };
    FilesystemCompactionRecoveryError::refused("finalize head.next", source).executing(
        completed,
        Action::Finalize(generation),
        boundary,
        effects,
    )
}
