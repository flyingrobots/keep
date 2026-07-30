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
    pub(super) fn unchecked_for_repository_tasks(
        lock: FilesystemWriterLock,
    ) -> std::io::Result<Self> {
        Self::unchecked(lock)
    }

    pub(super) fn into_lock(self) -> FilesystemWriterLock {
        self.lock
    }

    pub(super) fn into_parts(self) -> (FilesystemWriterLock, FilesystemRootIdentity) {
        (self.lock, self.root_identity)
    }

    #[cfg(any(test, feature = "repository-tasks"))]
    fn unchecked(lock: FilesystemWriterLock) -> std::io::Result<Self> {
        let directory = lock.clone_directory()?;
        let root_identity = super::filesystem_platform_profile::root_identity(&directory)?;
        Ok(Self::initialized(lock, root_identity))
    }
}
