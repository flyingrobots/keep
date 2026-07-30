//! Deterministic storage double for stage-discard orchestration.

use std::io;

use keep::{
    RecoveryStageDiscardOutcome, RecoveryStageDiscardStorage, RecoveryStageDiscardStorageError,
    RecoveryStageEvidence, RecoveryStageParent,
};

/// One semantic operation observed by the deterministic storage double.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Operation {
    /// Exact-evidence removal attempt.
    Remove(RecoveryStageEvidence),
    /// Name-selected parent synchronization attempt.
    Synchronize(RecoveryStageParent),
}

/// In-memory stage-discard storage with deterministic failure injection.
pub struct StageDiscardDouble {
    present: Option<RecoveryStageEvidence>,
    operations: Vec<Operation>,
    fail_synchronizations: usize,
}

impl StageDiscardDouble {
    /// Creates a double with the supplied canonical-stage observation.
    pub const fn new(present: Option<RecoveryStageEvidence>) -> Self {
        Self {
            present,
            operations: Vec::new(),
            fail_synchronizations: 0,
        }
    }

    /// Configures the next parent synchronization to fail exactly once.
    #[must_use]
    pub const fn fail_next_synchronization(mut self) -> Self {
        self.fail_synchronizations = 1;
        self
    }

    /// Returns the stage evidence currently retained by the double.
    pub const fn present(&self) -> Option<RecoveryStageEvidence> {
        self.present
    }

    /// Returns every semantic storage operation in call order.
    pub fn operations(&self) -> &[Operation] {
        &self.operations
    }
}

impl RecoveryStageDiscardStorage for StageDiscardDouble {
    fn remove_if_matching(
        &mut self,
        expected: RecoveryStageEvidence,
    ) -> Result<RecoveryStageDiscardOutcome, RecoveryStageDiscardStorageError> {
        self.operations.push(Operation::Remove(expected));
        match self.present {
            None => Ok(RecoveryStageDiscardOutcome::AlreadyAbsent),
            Some(observed) if observed == expected => {
                self.present = None;
                Ok(RecoveryStageDiscardOutcome::Removed)
            }
            Some(observed) => {
                Err(RecoveryStageDiscardStorageError::EvidenceMismatch { expected, observed })
            }
        }
    }

    fn synchronize_parent(&mut self, parent: RecoveryStageParent) -> io::Result<()> {
        self.operations.push(Operation::Synchronize(parent));
        if self.fail_synchronizations == 0 {
            return Ok(());
        }
        self.fail_synchronizations = self
            .fail_synchronizations
            .checked_sub(1)
            .ok_or_else(|| io::Error::other("synchronization counter underflow"))?;
        Err(io::Error::other("injected parent synchronization failure"))
    }
}
