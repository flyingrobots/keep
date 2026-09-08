//! This module owns the storage port for exact next-head finalization.

use std::io;

use super::{
    RecoveryNextHeadFinalizationReadiness, RecoveryNextHeadFinalizationRequest,
    RecoveryNextHeadFinalizationStorageError,
};

/// Semantic storage operations required by recovery head finalization.
///
/// Implementations must retain writer authority throughout execution.
/// Verification must re-open named paths without following links and establish
/// either the exact expected-current/candidate relationship or an exact
/// already-finalized candidate. Replacement must be atomic. The orchestration
/// layer owns operation order and receipt timing.
pub trait RecoveryNextHeadFinalizationStorage {
    /// Revalidates durable current state and the complete candidate view.
    ///
    /// # Errors
    ///
    /// Returns a source-preserving evidence, transition, verification, or
    /// storage failure.
    fn verify_current(
        &mut self,
        request: RecoveryNextHeadFinalizationRequest,
    ) -> Result<RecoveryNextHeadFinalizationReadiness, RecoveryNextHeadFinalizationStorageError>;

    /// Synchronizes the exact complete candidate before head replacement.
    ///
    /// Implementations must bind the synchronized file to the request evidence
    /// and revalidate its complete transitive view after synchronization.
    ///
    /// # Errors
    ///
    /// Returns a source-preserving evidence, synchronization, verification, or
    /// storage failure.
    fn synchronize_candidate(
        &mut self,
        request: RecoveryNextHeadFinalizationRequest,
    ) -> Result<(), RecoveryNextHeadFinalizationStorageError>;

    /// Atomically replaces durable `HEAD` with the exact candidate.
    ///
    /// # Errors
    ///
    /// Returns a source-preserving evidence or storage failure.
    fn replace_head(
        &mut self,
        request: RecoveryNextHeadFinalizationRequest,
    ) -> Result<(), RecoveryNextHeadFinalizationStorageError>;

    /// Synchronizes the root directory after replacement or retry admission.
    ///
    /// # Errors
    ///
    /// Returns the exact root-directory synchronization failure.
    fn synchronize_root(&mut self) -> io::Result<()>;
}
