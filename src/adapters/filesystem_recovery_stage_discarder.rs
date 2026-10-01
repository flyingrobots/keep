//! This module owns pinned writer authority for filesystem stage discard.

use std::path::Path;

#[cfg(any(test, feature = "repository-tasks"))]
use cap_std::ambient_authority;
use cap_std::fs::Dir;

use super::{
    FilesystemRecoveryInventoryReader, FilesystemRecoveryStageDiscardOpenError,
    FilesystemWriterLock, filesystem_platform_profile,
};
#[cfg(any(test, feature = "repository-tasks"))]
use super::{RecoveryInventoryError, RecoveryInventoryOperation, RecoveryNamespace};

/// Writer-authorized pinned filesystem adapter for exact stage discard.
///
/// Opening proves the supported platform, pins and exclusively locks the store
/// root and `writer.lock`, then pins all three protocol child directories
/// without following links. The synchronous adapter may block on filesystem
/// I/O and retains writer authority until dropped.
#[must_use]
pub struct FilesystemRecoveryStageDiscarder {
    pub(super) inventory: FilesystemRecoveryInventoryReader,
    _authority: FilesystemWriterLock,
}

impl FilesystemRecoveryStageDiscarder {
    /// Opens an initialized supported store for explicit stage discard.
    ///
    /// The call performs no protocol mutation.
    ///
    /// # Errors
    ///
    /// Returns [`FilesystemRecoveryStageDiscardOpenError`] on platform,
    /// writer-authority, root-clone, or namespace admission failure.
    pub fn open(store_root: &Path) -> Result<Self, FilesystemRecoveryStageDiscardOpenError> {
        let root = filesystem_platform_profile::open(store_root)
            .map_err(|source| FilesystemRecoveryStageDiscardOpenError::Platform { source })?;
        Self::from_root(root)
    }

    /// Opens a completely migrated version-two store for explicit stage
    /// discard: the same protocol over the same fixed stage names, with the
    /// version-two root entries admitted as inert.
    ///
    /// # Errors
    ///
    /// As [`Self::open`].
    pub fn open_version_two(
        store_root: &Path,
    ) -> Result<Self, FilesystemRecoveryStageDiscardOpenError> {
        let root = filesystem_platform_profile::open_version_two(store_root)
            .map_err(|source| FilesystemRecoveryStageDiscardOpenError::Platform { source })?;
        Self::from_root_with(root, true)
    }

    #[cfg(test)]
    pub(super) fn open_unchecked_for_tests(
        store_root: &Path,
    ) -> Result<Self, FilesystemRecoveryStageDiscardOpenError> {
        Self::open_unchecked(store_root)
    }

    #[cfg(test)]
    pub(super) fn open_unchecked_version_two_for_tests(
        store_root: &Path,
    ) -> Result<Self, FilesystemRecoveryStageDiscardOpenError> {
        let root = Dir::open_ambient_dir(store_root, ambient_authority()).map_err(|source| {
            FilesystemRecoveryStageDiscardOpenError::Namespace {
                source: RecoveryInventoryError::io(
                    RecoveryNamespace::Root,
                    RecoveryInventoryOperation::OpenNamespace,
                    source,
                ),
            }
        })?;
        Self::from_root_with(root, true)
    }

    /// Opens repository crash-test storage without the production platform
    /// profile probe.
    ///
    /// # Errors
    ///
    /// Returns the exact root, writer-lock, or namespace admission failure.
    #[cfg(feature = "repository-tasks")]
    #[doc(hidden)]
    pub fn open_unchecked_for_repository_tasks(
        store_root: &Path,
    ) -> Result<Self, FilesystemRecoveryStageDiscardOpenError> {
        Self::open_unchecked(store_root)
    }

    #[cfg(any(test, feature = "repository-tasks"))]
    fn open_unchecked(store_root: &Path) -> Result<Self, FilesystemRecoveryStageDiscardOpenError> {
        let root = Dir::open_ambient_dir(store_root, ambient_authority()).map_err(|source| {
            FilesystemRecoveryStageDiscardOpenError::Namespace {
                source: RecoveryInventoryError::io(
                    RecoveryNamespace::Root,
                    RecoveryInventoryOperation::OpenNamespace,
                    source,
                ),
            }
        })?;
        Self::from_root(root)
    }

    fn from_root(root: Dir) -> Result<Self, FilesystemRecoveryStageDiscardOpenError> {
        Self::from_root_with(root, false)
    }

    fn from_root_with(
        root: Dir,
        version_two: bool,
    ) -> Result<Self, FilesystemRecoveryStageDiscardOpenError> {
        let authority = FilesystemWriterLock::try_acquire_in(root)
            .map_err(|source| FilesystemRecoveryStageDiscardOpenError::WriterLock { source })?;
        let inventory_root = authority
            .clone_directory()
            .map_err(|source| FilesystemRecoveryStageDiscardOpenError::CloneRoot { source })?;
        let inventory = if version_two {
            FilesystemRecoveryInventoryReader::from_root_version_two(inventory_root)
        } else {
            FilesystemRecoveryInventoryReader::from_root(inventory_root)
        }
        .map_err(|source| FilesystemRecoveryStageDiscardOpenError::Namespace { source })?;
        Ok(Self {
            inventory,
            _authority: authority,
        })
    }
}
