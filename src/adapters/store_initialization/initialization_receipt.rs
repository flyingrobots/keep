//! This module owns the result of one completed direct initialization.

use super::StoreInitializationPrefix;
use crate::adapters::FilesystemStoreAdmission;

/// Writer authority over a store whose initialization has durably committed.
///
/// Returned only after the commit synchronization, the removal of
/// `FORMAT.next`, the final root synchronization, and complete store
/// admission of the committed root. The store holds no catalog `HEAD` until
/// the first publication.
#[must_use]
pub struct StoreInitialization {
    admission: FilesystemStoreAdmission,
    found: StoreInitializationPrefix,
}

impl StoreInitialization {
    pub(in crate::adapters) const fn new(
        admission: FilesystemStoreAdmission,
        found: StoreInitializationPrefix,
    ) -> Self {
        Self { admission, found }
    }

    /// The state the call found under writer authority.
    ///
    /// A fresh root reports [`StoreInitializationPrefix::Empty`]; any
    /// other value names the interrupted state the call resumed.
    pub const fn found(&self) -> StoreInitializationPrefix {
        self.found
    }

    /// Releases the retained writer authority for forward publication.
    pub fn into_admission(self) -> FilesystemStoreAdmission {
        self.admission
    }
}
