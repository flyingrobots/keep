//! Deterministic storage double for complete-stage recovery.

use std::io;

use keep::{
    RecoveryStageCompletionPool, RecoveryStageCompletionRequest, RecoveryStageCompletionStorage,
    RecoveryStageCompletionStorageError, RecoveryStageDiscardOutcome,
    RecoveryStageDiscardStorageError, RecoveryStageEvidence, RecoveryStagePoolOutcome,
    RecoveryStageSynchronizationOutcome,
};

/// One semantic operation observed by the deterministic storage double.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Operation {
    /// Exact stage verification and file synchronization.
    SynchronizeStage(RecoveryStageCompletionRequest),
    /// Stage-to-pool link or completed-pool admission.
    LinkOrAdmit(RecoveryStageCompletionRequest),
    /// Exact completed-pool verification.
    VerifyPool(RecoveryStageCompletionRequest),
    /// Selected immutable-pool synchronization.
    SynchronizePool(RecoveryStageCompletionPool),
    /// Exact-evidence stage removal.
    RemoveStage(RecoveryStageEvidence),
    /// Staging-directory synchronization.
    SynchronizeStaging,
}

/// In-memory complete-stage storage with deterministic failure injection.
pub struct StageCompletionDouble {
    stage: Option<RecoveryStageEvidence>,
    pool: Option<RecoveryStageCompletionRequest>,
    operations: Vec<Operation>,
    fail_stage_synchronizations: usize,
    fail_pool_synchronizations: usize,
    fail_staging_synchronizations: usize,
}

impl StageCompletionDouble {
    /// Creates a double with the supplied stage and pool observations.
    pub const fn new(
        stage: Option<RecoveryStageEvidence>,
        pool: Option<RecoveryStageCompletionRequest>,
    ) -> Self {
        Self {
            stage,
            pool,
            operations: Vec::new(),
            fail_stage_synchronizations: 0,
            fail_pool_synchronizations: 0,
            fail_staging_synchronizations: 0,
        }
    }

    /// Configures the next staged-file synchronization to fail once.
    #[must_use]
    pub const fn fail_next_stage_synchronization(mut self) -> Self {
        self.fail_stage_synchronizations = 1;
        self
    }

    /// Configures the next immutable-pool synchronization to fail once.
    #[must_use]
    pub const fn fail_next_pool_synchronization(mut self) -> Self {
        self.fail_pool_synchronizations = 1;
        self
    }

    /// Configures the next staging synchronization to fail once.
    #[must_use]
    pub const fn fail_next_staging_synchronization(mut self) -> Self {
        self.fail_staging_synchronizations = 1;
        self
    }

    /// Returns the current canonical stage evidence.
    pub const fn stage(&self) -> Option<RecoveryStageEvidence> {
        self.stage
    }

    /// Returns the current pooled request coordinate.
    pub const fn pool(&self) -> Option<RecoveryStageCompletionRequest> {
        self.pool
    }

    /// Returns every semantic operation in call order.
    pub fn operations(&self) -> &[Operation] {
        &self.operations
    }
}

impl RecoveryStageCompletionStorage for StageCompletionDouble {
    fn synchronize_stage_if_present(
        &mut self,
        request: RecoveryStageCompletionRequest,
    ) -> Result<RecoveryStageSynchronizationOutcome, RecoveryStageCompletionStorageError> {
        self.operations.push(Operation::SynchronizeStage(request));
        match self.stage {
            Some(observed) if observed == request.evidence() => {
                fail_once(
                    &mut self.fail_stage_synchronizations,
                    "injected stage synchronization failure",
                )
                .map_err(|source| RecoveryStageCompletionStorageError::Storage { source })?;
                Ok(RecoveryStageSynchronizationOutcome::Synchronized)
            }
            Some(observed) => Err(RecoveryStageCompletionStorageError::EvidenceMismatch {
                expected: request.evidence(),
                observed,
            }),
            None => Ok(RecoveryStageSynchronizationOutcome::AlreadyAbsent),
        }
    }

    fn link_stage_or_admit_pool(
        &mut self,
        request: RecoveryStageCompletionRequest,
    ) -> Result<RecoveryStagePoolOutcome, RecoveryStageCompletionStorageError> {
        self.operations.push(Operation::LinkOrAdmit(request));
        if self.pool.is_some() {
            return Ok(RecoveryStagePoolOutcome::AlreadyPresent);
        }
        match self.stage {
            Some(observed) if observed == request.evidence() => {
                self.pool = Some(request);
                Ok(RecoveryStagePoolOutcome::Linked)
            }
            Some(observed) => Err(RecoveryStageCompletionStorageError::EvidenceMismatch {
                expected: request.evidence(),
                observed,
            }),
            None => Err(RecoveryStageCompletionStorageError::Missing { request }),
        }
    }

    fn verify_pool(
        &mut self,
        request: RecoveryStageCompletionRequest,
    ) -> Result<(), RecoveryStageCompletionStorageError> {
        self.operations.push(Operation::VerifyPool(request));
        if self.pool == Some(request) {
            Ok(())
        } else {
            let observed = self.pool.map_or_else(
                || request.evidence(),
                RecoveryStageCompletionRequest::evidence,
            );
            Err(RecoveryStageCompletionStorageError::EvidenceMismatch {
                expected: request.evidence(),
                observed,
            })
        }
    }

    fn synchronize_pool(&mut self, pool: RecoveryStageCompletionPool) -> io::Result<()> {
        self.operations.push(Operation::SynchronizePool(pool));
        fail_once(
            &mut self.fail_pool_synchronizations,
            "injected pool synchronization failure",
        )
    }

    fn remove_stage_if_matching(
        &mut self,
        expected: RecoveryStageEvidence,
    ) -> Result<RecoveryStageDiscardOutcome, RecoveryStageDiscardStorageError> {
        self.operations.push(Operation::RemoveStage(expected));
        match self.stage {
            None => Ok(RecoveryStageDiscardOutcome::AlreadyAbsent),
            Some(observed) if observed == expected => {
                self.stage = None;
                Ok(RecoveryStageDiscardOutcome::Removed)
            }
            Some(observed) => {
                Err(RecoveryStageDiscardStorageError::EvidenceMismatch { expected, observed })
            }
        }
    }

    fn synchronize_staging(&mut self) -> io::Result<()> {
        self.operations.push(Operation::SynchronizeStaging);
        fail_once(
            &mut self.fail_staging_synchronizations,
            "injected staging synchronization failure",
        )
    }
}

fn fail_once(remaining: &mut usize, message: &'static str) -> io::Result<()> {
    if *remaining == 0 {
        return Ok(());
    }
    *remaining = remaining
        .checked_sub(1)
        .ok_or_else(|| io::Error::other("failure counter underflow"))?;
    Err(io::Error::other(message))
}
