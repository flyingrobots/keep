//! This module owns the storage port for exact truncated-stage discard.

use std::io;

use super::{
    RecoveryStageDiscardOutcome, RecoveryStageDiscardStorageError, RecoveryStageEvidence,
    RecoveryStageParent,
};

/// Semantic storage operations required by explicit truncated-stage discard.
///
/// The implementation must retain writer authority. Removal must select the
/// canonical name from `expected.stage()`, reopen without following links,
/// bound the complete read by that stage's protocol maximum, reverify exact
/// length, fingerprint, namespace identity, and regular-file type, and refuse
/// disagreement without mutation. An absent canonical name is an idempotent
/// input. The orchestration layer owns operation ordering and receipt timing.
pub trait RecoveryStageDiscardStorage {
    /// Removes the exact stage or reports that its canonical name is absent.
    ///
    /// # Errors
    ///
    /// Returns a typed evidence mismatch without mutation or preserves the
    /// exact storage error from reopen, verification, or removal.
    fn remove_if_matching(
        &mut self,
        expected: RecoveryStageEvidence,
    ) -> Result<RecoveryStageDiscardOutcome, RecoveryStageDiscardStorageError>;

    /// Synchronizes the protocol-selected parent.
    ///
    /// # Errors
    ///
    /// Returns the exact parent-directory synchronization failure.
    fn synchronize_parent(&mut self, parent: RecoveryStageParent) -> io::Result<()>;
}
