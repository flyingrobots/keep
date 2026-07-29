//! This module owns the storage port for exact complete-stage recovery.

use std::io;

use super::{
    RecoveryStageCompletionPool, RecoveryStageCompletionRequest, RecoveryStageDiscardOutcome,
    RecoveryStageDiscardStorageError, RecoveryStageEvidence, RecoveryStagePoolOutcome,
    RecoveryStageSynchronizationOutcome,
};

/// Semantic storage operations required by complete-stage recovery.
///
/// Implementations must retain writer authority throughout execution. A link
/// must never replace a pool entry. An existing entry is input to exact
/// verification, not proof by name. Stage and pool reads must reopen without
/// following links, remain bounded by the selected protocol, and verify exact
/// evidence and semantic coordinates before mutation. The orchestration layer
/// owns operation order and receipt timing.
pub trait RecoveryStageCompletionStorage {
    /// Verifies and synchronizes the exact stage when it remains present.
    ///
    /// An absent stage is an idempotent input only when the subsequent
    /// link-or-admit operation can select an existing pool coordinate.
    ///
    /// # Errors
    ///
    /// Returns a source-preserving evidence, verification, or staged-file
    /// synchronization failure.
    fn synchronize_stage_if_present(
        &mut self,
        request: RecoveryStageCompletionRequest,
    ) -> io::Result<RecoveryStageSynchronizationOutcome>;

    /// Links an exact stage or admits an already-present pool coordinate.
    ///
    /// # Errors
    ///
    /// Returns a source-preserving storage error when neither an exact stage
    /// nor an existing pool coordinate can continue the request.
    fn link_stage_or_admit_pool(
        &mut self,
        request: RecoveryStageCompletionRequest,
    ) -> io::Result<RecoveryStagePoolOutcome>;

    /// Verifies that the selected pool entry exactly satisfies the request.
    ///
    /// # Errors
    ///
    /// Returns a source-preserving verification or storage error.
    fn verify_pool(&mut self, request: RecoveryStageCompletionRequest) -> io::Result<()>;

    /// Synchronizes the selected immutable-pool directory.
    ///
    /// # Errors
    ///
    /// Returns the exact directory synchronization failure.
    fn synchronize_pool(&mut self, pool: RecoveryStageCompletionPool) -> io::Result<()>;

    /// Removes the exact stage or reports that its canonical name is absent.
    ///
    /// # Errors
    ///
    /// Returns a typed evidence mismatch without mutation or preserves the
    /// exact storage error from reopen, verification, or removal.
    fn remove_stage_if_matching(
        &mut self,
        expected: RecoveryStageEvidence,
    ) -> Result<RecoveryStageDiscardOutcome, RecoveryStageDiscardStorageError>;

    /// Synchronizes the staging directory after exact removal or absent retry.
    ///
    /// # Errors
    ///
    /// Returns the exact staging-directory synchronization failure.
    fn synchronize_staging(&mut self) -> io::Result<()>;
}
