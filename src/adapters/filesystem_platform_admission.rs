//! This module owns proof that a filesystem root passed platform admission.

use super::FilesystemWriterLock;

/// Exclusive writer authority over a platform-admitted filesystem root.
///
/// Fields are private so only Keep's initialization and platform-admission
/// boundary can create production values. That boundary remains intentionally
/// absent until issue #17 supplies its crash-tested implementation.
#[must_use]
pub struct FilesystemPlatformAdmission {
    lock: FilesystemWriterLock,
}

impl FilesystemPlatformAdmission {
    #[cfg(test)]
    pub(super) const fn unchecked_for_tests(lock: FilesystemWriterLock) -> Self {
        Self { lock }
    }

    pub(super) fn into_lock(self) -> FilesystemWriterLock {
        self.lock
    }
}
