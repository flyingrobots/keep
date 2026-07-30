//! This module owns pinned writer authority for filesystem segment continuation.

use std::path::Path;

use super::{FilesystemRecoverySegmentResumeOpenError, FilesystemRecoveryStageDiscarder};

/// Writer-authorized pinned filesystem adapter for reusable segment recovery.
///
/// Opening proves the supported platform, pins and exclusively locks the store
/// root and `writer.lock`, then pins all three protocol child directories
/// without following links. Execution consumes this value so the returned
/// writable stage retains that authority until it is sealed or dropped.
#[must_use]
pub struct FilesystemRecoverySegmentResumer {
    pub(super) discarder: FilesystemRecoveryStageDiscarder,
    #[cfg(test)]
    pub(super) before_handoff: Option<Box<dyn FnOnce()>>,
}

impl FilesystemRecoverySegmentResumer {
    /// Opens an initialized supported store for explicit segment continuation.
    ///
    /// The call performs no protocol mutation.
    ///
    /// # Errors
    ///
    /// Returns [`FilesystemRecoverySegmentResumeOpenError`] on platform,
    /// writer-authority, root-clone, or namespace admission failure.
    pub fn open(store_root: &Path) -> Result<Self, FilesystemRecoverySegmentResumeOpenError> {
        FilesystemRecoveryStageDiscarder::open(store_root)
            .map(Self::from_discarder)
            .map_err(Into::into)
    }

    const fn from_discarder(discarder: FilesystemRecoveryStageDiscarder) -> Self {
        Self {
            discarder,
            #[cfg(test)]
            before_handoff: None,
        }
    }

    #[cfg(test)]
    pub(super) fn open_unchecked_for_tests(
        store_root: &Path,
    ) -> Result<Self, FilesystemRecoverySegmentResumeOpenError> {
        FilesystemRecoveryStageDiscarder::open_unchecked_for_tests(store_root)
            .map(Self::from_discarder)
            .map_err(Into::into)
    }

    #[cfg(test)]
    pub(super) fn open_unchecked_for_tests_before_handoff<F>(
        store_root: &Path,
        before_handoff: F,
    ) -> Result<Self, FilesystemRecoverySegmentResumeOpenError>
    where
        F: FnOnce() + 'static,
    {
        let mut resumer = Self::open_unchecked_for_tests(store_root)?;
        resumer.before_handoff = Some(Box::new(before_handoff));
        Ok(resumer)
    }
}
