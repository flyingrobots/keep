//! This module owns pinned writer authority for filesystem stage discard.

use super::{
    FilesystemRecoveryInventoryReader, FilesystemRecoveryStageDiscardOpenError,
    FilesystemVersionTwoRecoveryAdmission, FilesystemWriterLock,
};

/// Writer-authorized pinned filesystem adapter for exact stage discard.
///
/// Opening consumes recovery admission, which proved the supported platform,
/// the exact namespace, and the `FORMAT` marker under the exclusive writer
/// lock; it then pins all three protocol child directories without following
/// links. The synchronous adapter may block on filesystem I/O and retains
/// writer authority until dropped.
#[must_use]
pub struct FilesystemRecoveryStageDiscarder {
    pub(super) inventory: FilesystemRecoveryInventoryReader,
    _authority: FilesystemWriterLock,
}

impl FilesystemRecoveryStageDiscarder {
    /// Pins a committed store for explicit stage discard.
    ///
    /// The admission already holds writer authority over a root that passed
    /// recovery admission; this call pins the catalog-protocol directories
    /// under it and performs no protocol mutation. The retained retention
    /// capabilities are released: discard never changes `HEAD`.
    ///
    /// # Errors
    ///
    /// Returns [`FilesystemRecoveryStageDiscardOpenError`] on root-clone or
    /// namespace pinning failure.
    pub fn open_version_two(
        admission: FilesystemVersionTwoRecoveryAdmission,
    ) -> Result<Self, FilesystemRecoveryStageDiscardOpenError> {
        let (lock, _retention, _roots, _manifests) = admission.into_parts();
        Self::from_version_two_lock(lock)
    }

    pub(super) fn from_version_two_lock(
        authority: FilesystemWriterLock,
    ) -> Result<Self, FilesystemRecoveryStageDiscardOpenError> {
        let inventory_root = authority
            .clone_directory()
            .map_err(|source| FilesystemRecoveryStageDiscardOpenError::CloneRoot { source })?;
        let inventory = FilesystemRecoveryInventoryReader::from_version_two_root(inventory_root)
            .map_err(|source| FilesystemRecoveryStageDiscardOpenError::Namespace { source })?;
        Ok(Self {
            inventory,
            _authority: authority,
        })
    }
}
