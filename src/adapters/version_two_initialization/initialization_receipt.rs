//! This module owns the result of one completed direct initialization.

use super::VersionTwoInitializationPrefix;
use crate::adapters::FilesystemVersionTwoAdmission;

/// Writer authority over a store whose initialization has durably committed.
///
/// Returned only after the commit synchronization, the removal of
/// `FORMAT.next`, the final root synchronization, and complete version-two
/// admission of the committed root. The store holds no catalog `HEAD` until
/// the first publication.
#[must_use]
pub struct VersionTwoInitialization {
    admission: FilesystemVersionTwoAdmission,
    found: VersionTwoInitializationPrefix,
}

impl VersionTwoInitialization {
    pub(in crate::adapters) const fn new(
        admission: FilesystemVersionTwoAdmission,
        found: VersionTwoInitializationPrefix,
    ) -> Self {
        Self { admission, found }
    }

    /// The state the call found under writer authority.
    ///
    /// A fresh root reports [`VersionTwoInitializationPrefix::Empty`]; any
    /// other value names the interrupted state the call resumed.
    pub const fn found(&self) -> VersionTwoInitializationPrefix {
        self.found
    }

    /// Releases the retained writer authority for forward publication.
    pub fn into_admission(self) -> FilesystemVersionTwoAdmission {
        self.admission
    }
}
