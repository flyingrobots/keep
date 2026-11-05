//! This module owns pinned writer authority for filesystem segment continuation.

use super::{
    FilesystemRecoveryAdmission, FilesystemRecoverySegmentResumeOpenError,
    FilesystemRecoveryStageDiscarder,
};

/// Writer-authorized pinned filesystem adapter for reusable segment recovery.
///
/// Opening consumes recovery admission, which holds the exclusive writer lock
/// over an admitted root, then pins all three protocol child directories
/// without following links. Execution consumes this value so the returned
/// writable stage retains that authority until it is sealed or dropped.
#[must_use]
pub struct FilesystemRecoverySegmentResumer {
    pub(super) discarder: FilesystemRecoveryStageDiscarder,
    #[cfg(test)]
    pub(super) before_handoff: Option<Box<dyn FnOnce()>>,
}

impl FilesystemRecoverySegmentResumer {
    /// Pins a committed store for explicit segment continuation.
    ///
    /// The call performs no protocol mutation.
    ///
    /// # Errors
    ///
    /// Returns [`FilesystemRecoverySegmentResumeOpenError`] on root-clone or
    /// namespace pinning failure.
    pub fn open(
        admission: FilesystemRecoveryAdmission,
    ) -> Result<Self, FilesystemRecoverySegmentResumeOpenError> {
        FilesystemRecoveryStageDiscarder::open(admission)
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
    pub(super) fn open_before_handoff_for_tests<F>(
        admission: FilesystemRecoveryAdmission,
        before_handoff: F,
    ) -> Result<Self, FilesystemRecoverySegmentResumeOpenError>
    where
        F: FnOnce() + 'static,
    {
        let mut resumer = Self::open(admission)?;
        resumer.before_handoff = Some(Box::new(before_handoff));
        Ok(resumer)
    }
}
