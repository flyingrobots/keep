//! This module owns pinned writer authority for filesystem head finalization.

use cap_std::fs::Dir;

use super::retention::FilesystemRetainedEvidence;
use super::{
    CatalogRestartPolicy, FilesystemRecoveryNextHeadFinalizationOpenError,
    FilesystemRecoveryStageDiscardOpenError, FilesystemRecoveryStageDiscarder,
    FilesystemVersionTwoRecoveryAdmission, RecoveryStageParent,
};

/// Writer-authorized pinned filesystem adapter for exact next-head finalization.
///
/// Opening consumes recovery admission, which holds the exclusive writer lock
/// over an admitted root, then pins all three protocol child directories
/// without following links. The synchronous adapter may block on bounded
/// filesystem I/O and retains writer authority until dropped.
///
/// The finalizer also retains the admitted retention capabilities and,
/// immediately before every candidate verification that can precede `HEAD`
/// replacement, verifies every current retained closure against the staged
/// candidate with the same admission the forward catalog gate runs. A
/// candidate that would lose retained content refuses and leaves `head.next`
/// in place.
#[must_use]
pub struct FilesystemRecoveryNextHeadFinalizer {
    pub(super) discarder: FilesystemRecoveryStageDiscarder,
    pub(super) policy: CatalogRestartPolicy,
    pub(super) retention: FilesystemRetainedEvidence,
}

impl FilesystemRecoveryNextHeadFinalizer {
    /// Pins a committed store for gated head finalization.
    ///
    /// The call performs no protocol mutation and reads no record. Execution
    /// verifies every retained closure named by the current retention head
    /// against the candidate before `HEAD` can change.
    ///
    /// # Errors
    ///
    /// Returns [`FilesystemRecoveryNextHeadFinalizationOpenError`] on root-clone
    /// or namespace pinning failure.
    pub fn open_version_two(
        admission: FilesystemVersionTwoRecoveryAdmission,
        policy: CatalogRestartPolicy,
    ) -> Result<Self, FilesystemRecoveryNextHeadFinalizationOpenError> {
        let (lock, retention, roots, manifests) = admission.into_parts();
        let root = lock
            .clone_directory()
            .map_err(|source| FilesystemRecoveryStageDiscardOpenError::CloneRoot { source })?;
        let discarder = FilesystemRecoveryStageDiscarder::from_version_two_lock(lock)?;
        Ok(Self {
            discarder,
            policy,
            retention: FilesystemRetainedEvidence::new(root, retention, roots, manifests),
        })
    }

    pub(super) const fn root(&self) -> &Dir {
        self.discarder
            .inventory
            .parent_directory(RecoveryStageParent::Root)
    }
}
