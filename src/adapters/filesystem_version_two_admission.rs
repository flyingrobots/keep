//! This module owns proof that a filesystem root passed version-two admission.

use std::path::Path;

#[cfg(test)]
use cap_std::ambient_authority;
use cap_std::fs::Dir;

use super::filesystem_root_identity::FilesystemRootIdentity;
use super::{
    FilesystemPlatformAdmissionError, FilesystemWriterLock, filesystem_initialization_namespace,
    filesystem_platform_profile, filesystem_version_two_records,
};

/// Exclusive writer authority over a completely migrated version-two root.
///
/// This type is deliberately distinct from
/// [`FilesystemPlatformAdmission`](super::FilesystemPlatformAdmission): a
/// version-one publisher cannot consume it, so version-one catalog publication
/// can never run against a migrated root and leave residue no adapter admits.
/// Fields are private so only version-two admission can create values.
#[must_use]
pub struct FilesystemVersionTwoAdmission {
    lock: FilesystemWriterLock,
}

impl FilesystemVersionTwoAdmission {
    /// Reacquires writer authority over one completely migrated version-two store.
    ///
    /// The call mutates no protocol state. It admits the production platform
    /// for every version-two protocol directory, acquires the existing writer
    /// lock, requires the exact version-two root namespace, and jointly admits
    /// the marker, intent, and receipt records. Retention adapters perform
    /// content-level validation under the returned authority. The synchronous
    /// call may block on filesystem I/O.
    ///
    /// # Errors
    ///
    /// Returns [`FilesystemPlatformAdmissionError`] with the exact platform,
    /// writer-lock, namespace, or migration-record boundary and preserved
    /// source.
    pub fn reopen(store_root: &Path) -> Result<Self, FilesystemPlatformAdmissionError> {
        let root = filesystem_platform_profile::open_version_two(store_root)
            .map_err(|source| FilesystemPlatformAdmissionError::Platform { source })?;
        Self::admit(root)
    }

    #[cfg(test)]
    pub(super) fn reopen_unchecked_for_tests(
        store_root: &Path,
    ) -> Result<Self, FilesystemPlatformAdmissionError> {
        let root = Dir::open_ambient_dir(store_root, ambient_authority())
            .map_err(|source| FilesystemPlatformAdmissionError::Platform { source })?;
        Self::admit(root)
    }

    pub(super) fn into_lock(self) -> FilesystemWriterLock {
        self.lock
    }

    fn admit(root: Dir) -> Result<Self, FilesystemPlatformAdmissionError> {
        let lock = FilesystemWriterLock::try_acquire_in(root)
            .map_err(|source| FilesystemPlatformAdmissionError::WriterLock { source })?;
        let directory = lock
            .clone_directory()
            .map_err(|source| FilesystemPlatformAdmissionError::Namespace { source })?;
        filesystem_initialization_namespace::admit_version_two(&directory)
            .map_err(|source| FilesystemPlatformAdmissionError::Namespace { source })?;
        let _root_identity: FilesystemRootIdentity =
            filesystem_platform_profile::root_identity(&directory)
                .map_err(|source| FilesystemPlatformAdmissionError::Platform { source })?;
        filesystem_version_two_records::admit(&directory)
            .map_err(|source| FilesystemPlatformAdmissionError::MigrationRecord { source })?;
        Ok(Self { lock })
    }
}
