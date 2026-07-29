//! This module owns repository-only access to production initialization storage.

use std::io;
use std::path::Path;

use super::filesystem_initialization_storage::FilesystemInitializationStorage;
use super::{FilesystemWriterLock, StoreInitializationStorage};

/// Repository-tooling adapter over the production filesystem initialization
/// implementation.
///
/// This adapter bypasses the Linux ext4 profile probe so the process-death
/// harness can run on development hosts. It does not bypass namespace,
/// synchronization, writer-lock, or initialization protocol operations.
#[doc(hidden)]
pub struct RepositoryInitializationStorage {
    inner: FilesystemInitializationStorage,
}

impl RepositoryInitializationStorage {
    /// Opens `store_root` without applying the production platform profile.
    ///
    /// # Errors
    ///
    /// Returns the exact ambient root-open failure.
    pub fn admit_unchecked(store_root: &Path) -> io::Result<Self> {
        FilesystemInitializationStorage::admit_unchecked_for_repository_tasks(store_root)
            .map(|inner| Self { inner })
    }

    /// Consumes completed initialization storage and returns retained writer
    /// authority.
    ///
    /// # Errors
    ///
    /// Returns [`io::ErrorKind::Other`] when initialization did not acquire
    /// writer authority.
    pub fn into_writer_lock(self) -> io::Result<FilesystemWriterLock> {
        self.inner.into_lock()
    }
}

impl StoreInitializationStorage for RepositoryInitializationStorage {
    fn admit_platform(&mut self) -> io::Result<()> {
        self.inner.admit_platform()
    }

    fn open_and_lock_writer_file(&mut self) -> io::Result<()> {
        self.inner.open_and_lock_writer_file()
    }

    fn admit_staging_directory(&mut self) -> io::Result<()> {
        self.inner.admit_staging_directory()
    }

    fn admit_segment_pool_directory(&mut self) -> io::Result<()> {
        self.inner.admit_segment_pool_directory()
    }

    fn admit_catalog_pool_directory(&mut self) -> io::Result<()> {
        self.inner.admit_catalog_pool_directory()
    }

    fn synchronize_root(&mut self) -> io::Result<()> {
        self.inner.synchronize_root()
    }
}
