//! This module owns pinned writer authority for filesystem stage completion.

use std::path::Path;

use super::{FilesystemRecoveryStageCompletionOpenError, FilesystemRecoveryStageDiscarder};

/// Writer-authorized pinned filesystem adapter for exact stage completion.
///
/// Opening proves the supported platform, pins and exclusively locks the store
/// root and `writer.lock`, then pins all three protocol child directories
/// without following links. The synchronous adapter may block on filesystem
/// I/O and retains writer authority until dropped.
#[must_use]
pub struct FilesystemRecoveryStageCompleter {
    pub(super) discarder: FilesystemRecoveryStageDiscarder,
}

impl FilesystemRecoveryStageCompleter {
    /// Opens an initialized supported store for explicit stage completion.
    ///
    /// The call performs no protocol mutation.
    ///
    /// # Errors
    ///
    /// Returns [`FilesystemRecoveryStageCompletionOpenError`] on platform,
    /// writer-authority, root-clone, or namespace admission failure.
    pub fn open(store_root: &Path) -> Result<Self, FilesystemRecoveryStageCompletionOpenError> {
        FilesystemRecoveryStageDiscarder::open(store_root)
            .map(|discarder| Self { discarder })
            .map_err(Into::into)
    }

    #[cfg(test)]
    pub(super) fn open_unchecked_for_tests(
        store_root: &Path,
    ) -> Result<Self, FilesystemRecoveryStageCompletionOpenError> {
        FilesystemRecoveryStageDiscarder::open_unchecked_for_tests(store_root)
            .map(|discarder| Self { discarder })
            .map_err(Into::into)
    }
}
