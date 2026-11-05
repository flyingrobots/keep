//! This module owns pinned writer authority for filesystem stage completion.

use super::{
    FilesystemRecoveryAdmission, FilesystemRecoveryStageCompletionOpenError,
    FilesystemRecoveryStageDiscarder,
};

/// Writer-authorized pinned filesystem adapter for exact stage completion.
///
/// Opening consumes recovery admission, which holds the exclusive writer lock
/// over an admitted root, then pins all three protocol child directories
/// without following links. The synchronous adapter may block on filesystem
/// I/O and retains writer authority until dropped.
#[must_use]
pub struct FilesystemRecoveryStageCompleter {
    pub(super) discarder: FilesystemRecoveryStageDiscarder,
}

impl FilesystemRecoveryStageCompleter {
    /// Pins a committed store for explicit stage completion.
    ///
    /// The call performs no protocol mutation. Completion links a stage into
    /// an immutable pool and never changes `HEAD`.
    ///
    /// # Errors
    ///
    /// Returns [`FilesystemRecoveryStageCompletionOpenError`] on root-clone or
    /// namespace pinning failure.
    pub fn open(
        admission: FilesystemRecoveryAdmission,
    ) -> Result<Self, FilesystemRecoveryStageCompletionOpenError> {
        FilesystemRecoveryStageDiscarder::open(admission)
            .map(|discarder| Self { discarder })
            .map_err(Into::into)
    }
}
