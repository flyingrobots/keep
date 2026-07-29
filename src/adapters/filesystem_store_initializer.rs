//! This module owns production filesystem platform admission and initialization.

use std::path::Path;

use super::filesystem_initialization_storage::FilesystemInitializationStorage;
use super::{
    FilesystemPlatformAdmission, StoreInitializationError, StoreInitializationPhase,
    initialize_store,
};

impl FilesystemPlatformAdmission {
    /// Initializes or resumes one admitted filesystem store root.
    ///
    /// The production adapter currently admits only a writable,
    /// case-sensitive Linux ext4 root opened without symbolic links. It
    /// validates an empty or partial canonical namespace before mutation,
    /// retains the exclusive writer lock, and returns only after synchronizing
    /// the complete root namespace. The call is synchronous, allocates no
    /// content-sized memory, and may block on filesystem I/O.
    ///
    /// # Errors
    ///
    /// Returns [`StoreInitializationError::Io`] at the exact failed phase.
    /// Unsupported, read-only, aliased, unknown, or ambiguous platform state is
    /// refused before protocol directory creation.
    pub fn initialize(store_root: &Path) -> Result<Self, StoreInitializationError> {
        let storage = FilesystemInitializationStorage::admit(store_root).map_err(|source| {
            StoreInitializationError::io(StoreInitializationPhase::AdmitPlatform, source)
        })?;
        initialize_storage(storage)
    }

    #[cfg(test)]
    pub(super) fn initialize_unchecked_for_tests(
        store_root: &Path,
    ) -> Result<Self, StoreInitializationError> {
        let storage = FilesystemInitializationStorage::admit_unchecked_for_tests(store_root)
            .map_err(|source| {
                StoreInitializationError::io(StoreInitializationPhase::AdmitPlatform, source)
            })?;
        initialize_storage(storage)
    }
}

fn initialize_storage(
    mut storage: FilesystemInitializationStorage,
) -> Result<FilesystemPlatformAdmission, StoreInitializationError> {
    let _receipt = initialize_store(&mut storage)?;
    let lock = storage.into_lock().map_err(|source| {
        StoreInitializationError::io(StoreInitializationPhase::OpenAndLockWriterFile, source)
    })?;
    Ok(FilesystemPlatformAdmission::initialized(lock))
}
