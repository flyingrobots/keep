//! This module owns pinned writer authority for filesystem head finalization.

use std::path::Path;

use cap_std::fs::Dir;

use super::retention::FilesystemRetainedEvidence;
use super::{
    CatalogRestartPolicy, FilesystemRecoveryNextHeadFinalizationOpenError,
    FilesystemRecoveryStageDiscardOpenError, FilesystemRecoveryStageDiscarder,
    FilesystemVersionTwoRecoveryAdmission, RecoveryStageParent,
};

/// Writer-authorized pinned filesystem adapter for exact next-head finalization.
///
/// Opening proves the supported platform, pins and exclusively locks the store
/// root and `writer.lock`, then pins all three protocol child directories
/// without following links. The synchronous adapter may block on bounded
/// filesystem I/O and retains writer authority until dropped.
///
/// A finalizer opened over a version-two store also retains the admitted
/// retention capabilities and, immediately before every candidate
/// verification that can precede `HEAD` replacement, verifies every current
/// retained closure against the staged candidate with the same admission the
/// forward catalog gate runs. A candidate that would lose retained content
/// refuses and leaves `head.next` in place.
#[must_use]
pub struct FilesystemRecoveryNextHeadFinalizer {
    pub(super) discarder: FilesystemRecoveryStageDiscarder,
    pub(super) policy: CatalogRestartPolicy,
    pub(super) retention: FinalizationRetention,
}

/// Retention evidence a finalizer must preserve, by store format.
pub(super) enum FinalizationRetention {
    /// A version-one store has no retention state to preserve.
    VersionOne,
    /// A version-two store's retained closures gate every replacement.
    VersionTwo(FilesystemRetainedEvidence),
}

impl FilesystemRecoveryNextHeadFinalizer {
    /// Opens an initialized supported store for explicit head finalization.
    ///
    /// The call performs no protocol mutation and does not read `HEAD` or
    /// `head.next`.
    ///
    /// # Errors
    ///
    /// Returns [`FilesystemRecoveryNextHeadFinalizationOpenError`] on platform,
    /// writer-authority, root-clone, or namespace admission failure.
    pub fn open(
        store_root: &Path,
        policy: CatalogRestartPolicy,
    ) -> Result<Self, FilesystemRecoveryNextHeadFinalizationOpenError> {
        FilesystemRecoveryStageDiscarder::open(store_root)
            .map(|discarder| Self::version_one(discarder, policy))
            .map_err(Into::into)
    }

    /// Pins a committed version-two store for gated head finalization.
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
        let evidence = FilesystemRetainedEvidence::new(root, retention, roots, manifests);
        Ok(Self {
            discarder,
            policy,
            retention: FinalizationRetention::VersionTwo(evidence),
        })
    }

    #[cfg(test)]
    pub(super) fn open_unchecked_for_tests(
        store_root: &Path,
        policy: CatalogRestartPolicy,
    ) -> Result<Self, FilesystemRecoveryNextHeadFinalizationOpenError> {
        FilesystemRecoveryStageDiscarder::open_unchecked_for_tests(store_root)
            .map(|discarder| Self::version_one(discarder, policy))
            .map_err(Into::into)
    }

    const fn version_one(
        discarder: FilesystemRecoveryStageDiscarder,
        policy: CatalogRestartPolicy,
    ) -> Self {
        Self {
            discarder,
            policy,
            retention: FinalizationRetention::VersionOne,
        }
    }

    pub(super) const fn root(&self) -> &Dir {
        self.discarder
            .inventory
            .parent_directory(RecoveryStageParent::Root)
    }
}
