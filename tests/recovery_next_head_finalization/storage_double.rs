//! Deterministic storage double for next-head finalization.

use std::io;

use keep::{
    RecoveryNextHeadFinalizationReadiness, RecoveryNextHeadFinalizationRequest,
    RecoveryNextHeadFinalizationStorage, RecoveryNextHeadFinalizationStorageError,
};

/// One semantic operation observed by the deterministic storage double.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Operation {
    /// Current-state and candidate-view verification.
    Verify(RecoveryNextHeadFinalizationRequest),
    /// Exact candidate-file synchronization.
    SynchronizeCandidate(RecoveryNextHeadFinalizationRequest),
    /// Atomic candidate-head replacement.
    Replace(RecoveryNextHeadFinalizationRequest),
    /// Root-directory synchronization.
    SynchronizeRoot,
}

/// In-memory next-head storage with deterministic failure injection.
pub struct NextHeadDouble {
    readiness: RecoveryNextHeadFinalizationReadiness,
    operations: Vec<Operation>,
    fail_verifications: usize,
    fail_candidate_synchronizations: usize,
    fail_replacements: usize,
    fail_synchronizations: usize,
}

impl NextHeadDouble {
    /// Creates a double with the supplied current-state readiness.
    pub const fn new(readiness: RecoveryNextHeadFinalizationReadiness) -> Self {
        Self {
            readiness,
            operations: Vec::new(),
            fail_verifications: 0,
            fail_candidate_synchronizations: 0,
            fail_replacements: 0,
            fail_synchronizations: 0,
        }
    }

    /// Configures the next candidate synchronization to fail once.
    #[must_use]
    pub const fn fail_next_candidate_synchronization(mut self) -> Self {
        self.fail_candidate_synchronizations = 1;
        self
    }

    /// Configures the next verification to fail once.
    #[must_use]
    pub const fn fail_next_verification(mut self) -> Self {
        self.fail_verifications = 1;
        self
    }

    /// Configures the next atomic replacement to fail once.
    #[must_use]
    pub const fn fail_next_replacement(mut self) -> Self {
        self.fail_replacements = 1;
        self
    }

    /// Configures the next root synchronization to fail once.
    #[must_use]
    pub const fn fail_next_synchronization(mut self) -> Self {
        self.fail_synchronizations = 1;
        self
    }

    /// Returns every semantic storage operation in call order.
    pub fn operations(&self) -> &[Operation] {
        &self.operations
    }
}

impl RecoveryNextHeadFinalizationStorage for NextHeadDouble {
    fn verify_current(
        &mut self,
        request: RecoveryNextHeadFinalizationRequest,
    ) -> Result<RecoveryNextHeadFinalizationReadiness, RecoveryNextHeadFinalizationStorageError>
    {
        self.operations.push(Operation::Verify(request));
        fail_once(
            &mut self.fail_verifications,
            "injected current-state verification failure",
        )?;
        Ok(self.readiness)
    }

    fn synchronize_candidate(
        &mut self,
        request: RecoveryNextHeadFinalizationRequest,
    ) -> Result<(), RecoveryNextHeadFinalizationStorageError> {
        self.operations
            .push(Operation::SynchronizeCandidate(request));
        fail_once(
            &mut self.fail_candidate_synchronizations,
            "injected candidate synchronization failure",
        )
    }

    fn replace_head(
        &mut self,
        request: RecoveryNextHeadFinalizationRequest,
    ) -> Result<(), RecoveryNextHeadFinalizationStorageError> {
        self.operations.push(Operation::Replace(request));
        fail_once(
            &mut self.fail_replacements,
            "injected atomic replacement failure",
        )?;
        self.readiness = RecoveryNextHeadFinalizationReadiness::AlreadyFinalized;
        Ok(())
    }

    fn synchronize_root(&mut self) -> io::Result<()> {
        self.operations.push(Operation::SynchronizeRoot);
        if self.fail_synchronizations == 0 {
            return Ok(());
        }
        self.fail_synchronizations = self
            .fail_synchronizations
            .checked_sub(1)
            .ok_or_else(|| io::Error::other("synchronization counter underflow"))?;
        Err(io::Error::other("injected root synchronization failure"))
    }
}

fn fail_once(
    remaining: &mut usize,
    message: &'static str,
) -> Result<(), RecoveryNextHeadFinalizationStorageError> {
    if *remaining == 0 {
        return Ok(());
    }
    *remaining = remaining.checked_sub(1).ok_or_else(|| {
        RecoveryNextHeadFinalizationStorageError::Storage {
            source: io::Error::other("failure counter underflow"),
        }
    })?;
    Err(RecoveryNextHeadFinalizationStorageError::Storage {
        source: io::Error::other(message),
    })
}
