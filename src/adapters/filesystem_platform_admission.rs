//! This module owns proof that a filesystem root passed platform admission.

use super::FilesystemWriterLock;
use super::filesystem_root_identity::FilesystemRootIdentity;

/// Exclusive writer authority over a platform-admitted filesystem root.
///
/// Fields are private so only Keep's initialization and platform-admission
/// boundary can create production values.
#[must_use]
pub struct FilesystemPlatformAdmission {
    lock: FilesystemWriterLock,
    root_identity: FilesystemRootIdentity,
}

impl FilesystemPlatformAdmission {
    pub(super) const fn initialized(
        lock: FilesystemWriterLock,
        root_identity: FilesystemRootIdentity,
    ) -> Self {
        Self {
            lock,
            root_identity,
        }
    }

    #[cfg(test)]
    pub(super) fn unchecked_for_tests(lock: FilesystemWriterLock) -> std::io::Result<Self> {
        Self::unchecked(lock)
    }

    #[cfg(feature = "repository-tasks")]
    pub(super) fn from_repository_writer_lock(lock: FilesystemWriterLock) -> std::io::Result<Self> {
        // Writer capabilities may be O_PATH; profile ioctls require a readable
        // descriptor of that same pinned directory, without an ambient reopen.
        let pinned = lock.clone_directory()?;
        let directory = super::sync_capable_directory::open(&pinned, ".")?;
        let root_identity = super::filesystem_platform_profile::admit_locked_root(&directory)?;
        Ok(Self::initialized(lock, root_identity))
    }

    pub(super) fn into_lock(self) -> FilesystemWriterLock {
        self.lock
    }

    pub(super) fn into_parts(self) -> (FilesystemWriterLock, FilesystemRootIdentity) {
        (self.lock, self.root_identity)
    }

    /// Grants authority without platform admission only for private unit
    /// tests; the identity probe tolerates a kernel that reports no mount
    /// identity so the bypass does not require `STATX_MNT_ID`.
    #[cfg(test)]
    fn unchecked(lock: FilesystemWriterLock) -> std::io::Result<Self> {
        let directory = lock.clone_directory()?;
        let root_identity = super::filesystem_platform_profile::root_identity_lenient(&directory)?;
        Ok(Self::initialized(lock, root_identity))
    }
}
