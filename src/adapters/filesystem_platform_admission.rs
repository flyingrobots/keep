//! This module owns proof that a filesystem root passed platform admission.

use super::FilesystemWriterLock;

/// Exclusive writer authority over a platform-admitted filesystem root.
///
/// Fields are private so only Keep's initialization and platform-admission
/// boundary can create production values.
#[must_use]
pub struct FilesystemPlatformAdmission {
    lock: FilesystemWriterLock,
}

impl FilesystemPlatformAdmission {
    pub(super) const fn initialized(lock: FilesystemWriterLock) -> Self {
        Self { lock }
    }

    #[cfg(feature = "repository-tasks")]
    pub(super) fn from_repository_writer_lock(lock: FilesystemWriterLock) -> std::io::Result<Self> {
        // Writer capabilities may be O_PATH; profile ioctls require a readable
        // descriptor of that same pinned directory, without an ambient reopen.
        let pinned = lock.clone_directory()?;
        let directory = super::sync_capable_directory::open(&pinned, ".")?;
        super::filesystem_platform_profile::admit_locked_root(&directory)?;
        Ok(Self::initialized(lock))
    }

    pub(super) fn into_lock(self) -> FilesystemWriterLock {
        self.lock
    }
}
