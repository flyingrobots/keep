//! This module owns proof that a store root admits catalog-stage recovery.

use std::path::Path;

#[cfg(any(test, feature = "repository-tasks"))]
use cap_std::ambient_authority;
use cap_std::fs::Dir;

use super::filesystem_store_admission::{AdmittedStoreParts, admit_locked};
use super::{
    FilesystemStoreAdmissionError, filesystem_platform_profile, filesystem_store_namespace,
};

/// Exclusive writer authority over a committed root for stage recovery.
///
/// Admission is the writer admission with one difference: the
/// root may retain the catalog `head.next` that an interrupted publication
/// left. Every other law is the same: the production platform, the writer
/// lock, the exact namespace with its protocol directories, and the canonical
/// `FORMAT` marker. The admitted `retention`, `roots`, and `manifests` capabilities
/// are retained so recovery finalization verifies retained closures against
/// the directories that passed admission.
///
/// Every recovery tool that mutates a stage consumes this value, so recovery
/// never runs against a root without the marker.
#[must_use]
pub struct FilesystemRecoveryAdmission {
    parts: AdmittedStoreParts,
}

impl FilesystemRecoveryAdmission {
    /// Acquires writer authority over one committed store for stage recovery.
    ///
    /// The call mutates no protocol state and may block on filesystem I/O.
    ///
    /// # Errors
    ///
    /// Returns [`FilesystemStoreAdmissionError`] with the exact platform,
    /// writer-lock, namespace, or format-marker boundary and preserved
    /// source.
    pub fn reopen(store_root: &Path) -> Result<Self, FilesystemStoreAdmissionError> {
        let root = filesystem_platform_profile::open(store_root)
            .map_err(|source| FilesystemStoreAdmissionError::Platform { source })?;
        Self::admit(root)
    }

    /// Reopens a committed root without platform admission for repository tasks.
    ///
    /// Every namespace and record law still applies. Production
    /// callers use [`Self::reopen`].
    ///
    /// # Errors
    ///
    /// Returns [`FilesystemStoreAdmissionError`] exactly as [`Self::reopen`]
    /// does for every boundary after platform admission.
    #[cfg(any(test, feature = "repository-tasks"))]
    pub fn reopen_unchecked_for_repository_tasks(
        store_root: &Path,
    ) -> Result<Self, FilesystemStoreAdmissionError> {
        let root = Dir::open_ambient_dir(store_root, ambient_authority())
            .map_err(|source| FilesystemStoreAdmissionError::Platform { source })?;
        Self::admit(root)
    }

    /// Releases the writer lock and the three pinned retention capabilities.
    pub(super) fn into_parts(self) -> AdmittedStoreParts {
        self.parts
    }

    fn admit(root: Dir) -> Result<Self, FilesystemStoreAdmissionError> {
        admit_locked(
            root,
            filesystem_store_namespace::admit_store_with_catalog_stage,
        )
        .map(|parts| Self { parts })
    }
}
