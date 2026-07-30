//! This module owns pinned writer authority for filesystem head finalization.

use std::path::Path;

use cap_std::fs::Dir;

use super::{
    CatalogRestartPolicy, FilesystemRecoveryNextHeadFinalizationOpenError,
    FilesystemRecoveryStageDiscarder, RecoveryStageParent,
};

/// Writer-authorized pinned filesystem adapter for exact next-head finalization.
///
/// Opening proves the supported platform, pins and exclusively locks the store
/// root and `writer.lock`, then pins all three protocol child directories
/// without following links. The synchronous adapter may block on bounded
/// filesystem I/O and retains writer authority until dropped.
#[must_use]
pub struct FilesystemRecoveryNextHeadFinalizer {
    pub(super) discarder: FilesystemRecoveryStageDiscarder,
    pub(super) policy: CatalogRestartPolicy,
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
            .map(|discarder| Self { discarder, policy })
            .map_err(Into::into)
    }

    #[cfg(test)]
    pub(super) fn open_unchecked_for_tests(
        store_root: &Path,
        policy: CatalogRestartPolicy,
    ) -> Result<Self, FilesystemRecoveryNextHeadFinalizationOpenError> {
        FilesystemRecoveryStageDiscarder::open_unchecked_for_tests(store_root)
            .map(|discarder| Self { discarder, policy })
            .map_err(Into::into)
    }

    pub(super) const fn root(&self) -> &Dir {
        self.discarder
            .inventory
            .parent_directory(RecoveryStageParent::Root)
    }
}
