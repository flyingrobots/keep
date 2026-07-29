//! This module owns the storage port for store initialization.

use std::io;

/// Semantic storage operations required by ordered store initialization.
///
/// Implementations own platform proof, namespace technology, writer
/// authority, and idempotent create-or-admit behavior. The orchestration layer
/// owns only ordering and exact failure attribution.
pub trait StoreInitializationStorage {
    /// Proves the complete platform contract without mutating the namespace.
    ///
    /// # Errors
    ///
    /// Returns an I/O error when the platform cannot be proved admissible.
    fn admit_platform(&mut self) -> io::Result<()>;

    /// Creates or reopens the writer file and retains its exclusive lock.
    ///
    /// # Errors
    ///
    /// Returns an I/O error when the writer file or lock cannot be admitted.
    fn open_and_lock_writer_file(&mut self) -> io::Result<()>;

    /// Creates or verifies the exact `staging` directory.
    ///
    /// # Errors
    ///
    /// Returns an I/O error when the staging directory cannot be admitted.
    fn admit_staging_directory(&mut self) -> io::Result<()>;

    /// Creates or verifies the exact `segments` directory.
    ///
    /// # Errors
    ///
    /// Returns an I/O error when the segment-pool directory cannot be admitted.
    fn admit_segment_pool_directory(&mut self) -> io::Result<()>;

    /// Creates or verifies the exact `catalogs` directory.
    ///
    /// # Errors
    ///
    /// Returns an I/O error when the catalog-pool directory cannot be admitted.
    fn admit_catalog_pool_directory(&mut self) -> io::Result<()>;

    /// Synchronizes the store root after all canonical names exist.
    ///
    /// # Errors
    ///
    /// Returns an I/O error when root synchronization fails.
    fn synchronize_root(&mut self) -> io::Result<()>;
}
