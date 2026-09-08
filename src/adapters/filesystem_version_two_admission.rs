//! This module owns proof that a filesystem root passed version-two admission.

use std::path::Path;

use cap_fs_ext::DirExt;
#[cfg(test)]
use cap_std::ambient_authority;
use cap_std::fs::Dir;

use super::filesystem_root_identity::FilesystemRootIdentity;
pub(super) use super::filesystem_version_two_records::BoundRootIdentity;
use super::{
    FilesystemPlatformAdmissionError, FilesystemWriterLock, StoreRootIdentityCoordinate,
    filesystem_initialization_namespace, filesystem_platform_profile,
    filesystem_version_two_records,
};

/// Exclusive writer authority over a completely migrated version-two root.
///
/// This type is deliberately distinct from
/// [`FilesystemPlatformAdmission`](super::FilesystemPlatformAdmission): a
/// version-one publisher cannot consume it, so version-one catalog publication
/// can never run against a migrated root and leave residue no adapter admits.
/// Fields are private so only version-two admission can create values. The
/// admitted `retention`, `roots`, and `manifests` capabilities are retained,
/// so the authority built from this value operates on the directories that
/// passed admission and not on whatever those names resolve to later.
#[must_use]
pub struct FilesystemVersionTwoAdmission {
    lock: FilesystemWriterLock,
    retention: Dir,
    roots: Dir,
    manifests: Dir,
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

    /// Releases the writer lock and the three pinned retention capabilities.
    pub(super) fn into_parts(self) -> (FilesystemWriterLock, Dir, Dir, Dir) {
        (self.lock, self.retention, self.roots, self.manifests)
    }

    fn admit(root: Dir) -> Result<Self, FilesystemPlatformAdmissionError> {
        let lock = FilesystemWriterLock::try_acquire_in(root)
            .map_err(|source| FilesystemPlatformAdmissionError::WriterLock { source })?;
        let directory = lock
            .clone_directory()
            .map_err(|source| FilesystemPlatformAdmissionError::Namespace { source })?;
        filesystem_initialization_namespace::admit_version_two(&directory)
            .map_err(|source| FilesystemPlatformAdmissionError::Namespace { source })?;
        let observed = filesystem_platform_profile::root_identity(&directory)
            .map_err(|source| FilesystemPlatformAdmissionError::Platform { source })?;
        let bound = filesystem_version_two_records::admit(&directory)
            .map_err(|source| FilesystemPlatformAdmissionError::MigrationRecord { source })?;
        require_root_identity(bound, observed)?;
        let retention = pin(&directory, "retention")?;
        let roots = pin(&retention, "roots")?;
        let manifests = pin(&retention, "manifests")?;
        Ok(Self {
            lock,
            retention,
            roots,
            manifests,
        })
    }
}

/// Pins one admitted protocol directory without following links.
fn pin(parent: &Dir, name: &str) -> Result<Dir, FilesystemPlatformAdmissionError> {
    parent
        .open_dir_nofollow(name)
        .map_err(|source| FilesystemPlatformAdmissionError::Namespace { source })
}

/// Requires the reopened root to be the physical root the migration intent bound.
///
/// Device, mount, and file coordinates are compared exactly, as the migration
/// authority compares them before mutation. A relocated or restored store
/// refuses rather than receiving retention authority over a root whose intent
/// describes a different volume.
pub(super) fn require_root_identity(
    bound: BoundRootIdentity,
    observed: FilesystemRootIdentity,
) -> Result<(), FilesystemPlatformAdmissionError> {
    for (coordinate, expected, actual) in [
        (
            StoreRootIdentityCoordinate::Device,
            bound.device(),
            observed.device(),
        ),
        (
            StoreRootIdentityCoordinate::Mount,
            bound.mount(),
            observed.mount(),
        ),
        (
            StoreRootIdentityCoordinate::File,
            bound.file(),
            observed.file(),
        ),
    ] {
        if expected != actual {
            return Err(FilesystemPlatformAdmissionError::RootIdentityChanged {
                coordinate,
                expected,
                observed: actual,
            });
        }
    }
    Ok(())
}
