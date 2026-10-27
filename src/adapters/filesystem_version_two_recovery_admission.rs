//! This module owns proof that a version-two root admits catalog-stage recovery.

use std::path::Path;

#[cfg(any(test, feature = "repository-tasks"))]
use cap_std::ambient_authority;
use cap_std::fs::Dir;

use super::filesystem_version_two_admission::{AdmittedVersionTwoParts, admit_locked};
use super::{
    FilesystemPlatformAdmissionError, filesystem_initialization_namespace,
    filesystem_platform_profile,
};

/// Exclusive writer authority over a committed version-two root for stage recovery.
///
/// Admission is the version-two writer admission with one difference: the
/// root may retain the catalog `head.next` that an interrupted publication
/// left. Every other law is the same: the production platform, the writer
/// lock, the exact namespace with its protocol directories, and the canonical
/// `FORMAT` marker. The admitted `retention`, `roots`, and `manifests` capabilities
/// are retained so recovery finalization verifies retained closures against
/// the directories that passed admission.
///
/// Only the version-two recovery constructors consume this value; version-one
/// recovery keeps refusing a version-two root, and this admission refuses a
/// version-one root because the marker is required.
#[must_use]
pub struct FilesystemVersionTwoRecoveryAdmission {
    parts: AdmittedVersionTwoParts,
}

impl FilesystemVersionTwoRecoveryAdmission {
    /// Acquires writer authority over one committed store for stage recovery.
    ///
    /// The call mutates no protocol state and may block on filesystem I/O.
    ///
    /// # Errors
    ///
    /// Returns [`FilesystemPlatformAdmissionError`] with the exact platform,
    /// writer-lock, namespace, or format-marker boundary and preserved
    /// source.
    pub fn reopen(store_root: &Path) -> Result<Self, FilesystemPlatformAdmissionError> {
        let root = filesystem_platform_profile::open_version_two(store_root)
            .map_err(|source| FilesystemPlatformAdmissionError::Platform { source })?;
        Self::admit(root)
    }

    /// Reopens a committed root without platform admission for repository tasks.
    ///
    /// Every namespace and record law still applies. Production
    /// callers use [`Self::reopen`].
    ///
    /// # Errors
    ///
    /// Returns [`FilesystemPlatformAdmissionError`] exactly as [`Self::reopen`]
    /// does for every boundary after platform admission.
    #[cfg(any(test, feature = "repository-tasks"))]
    pub fn reopen_unchecked_for_repository_tasks(
        store_root: &Path,
    ) -> Result<Self, FilesystemPlatformAdmissionError> {
        let root = Dir::open_ambient_dir(store_root, ambient_authority())
            .map_err(|source| FilesystemPlatformAdmissionError::Platform { source })?;
        Self::admit(root)
    }

    /// Releases the writer lock and the three pinned retention capabilities.
    pub(super) fn into_parts(self) -> AdmittedVersionTwoParts {
        self.parts
    }

    fn admit(root: Dir) -> Result<Self, FilesystemPlatformAdmissionError> {
        admit_locked(
            root,
            filesystem_initialization_namespace::admit_version_two_with_catalog_stage,
        )
        .map(|parts| Self { parts })
    }
}
