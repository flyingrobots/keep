//! This module owns production filesystem platform admission and initialization.

use std::path::Path;

#[cfg(test)]
use cap_std::ambient_authority;
#[cfg(test)]
use cap_std::fs::Dir;

use super::filesystem_initialization_storage::FilesystemInitializationStorage;
use super::{
    FilesystemPlatformAdmission, FilesystemPlatformAdmissionError, FilesystemWriterLock,
    StoreInitializationError, StoreInitializationPhase, filesystem_initialization_namespace,
    filesystem_platform_profile, initialize_store,
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

    /// Reacquires writer authority over one completely published store.
    ///
    /// The call mutates no protocol state. It admits the production platform,
    /// acquires the existing writer lock, and requires the exact published root
    /// namespace: `writer.lock`, `staging`, `segments`, `catalogs`, and a
    /// regular `HEAD`. Publication and restart adapters perform content-level
    /// validation under the returned authority. The synchronous call may block
    /// on filesystem I/O.
    ///
    /// # Errors
    ///
    /// Returns [`FilesystemPlatformAdmissionError`] with the exact platform,
    /// writer-lock, or namespace boundary and preserved source.
    pub fn reopen(store_root: &Path) -> Result<Self, FilesystemPlatformAdmissionError> {
        let root = filesystem_platform_profile::open(store_root)
            .map_err(|source| FilesystemPlatformAdmissionError::Platform { source })?;
        reopen_root(root)
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

    #[cfg(test)]
    pub(super) fn reopen_unchecked_for_tests(
        store_root: &Path,
    ) -> Result<Self, FilesystemPlatformAdmissionError> {
        let root = Dir::open_ambient_dir(store_root, ambient_authority())
            .map_err(|source| FilesystemPlatformAdmissionError::Platform { source })?;
        reopen_root(root)
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

fn reopen_root(
    root: cap_std::fs::Dir,
) -> Result<FilesystemPlatformAdmission, FilesystemPlatformAdmissionError> {
    let lock = FilesystemWriterLock::try_acquire_in(root)
        .map_err(|source| FilesystemPlatformAdmissionError::WriterLock { source })?;
    let directory = lock
        .clone_directory()
        .map_err(|source| FilesystemPlatformAdmissionError::Namespace { source })?;
    filesystem_initialization_namespace::admit_published(&directory)
        .map_err(|source| FilesystemPlatformAdmissionError::Namespace { source })?;
    Ok(FilesystemPlatformAdmission::initialized(lock))
}
