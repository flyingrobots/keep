//! This module owns capability-relative filesystem recovery inventory.

use std::io;
use std::path::Path;

#[cfg(test)]
use cap_std::ambient_authority;
use cap_std::fs::Dir;

use super::{
    RecoveryEntryName, RecoveryInventory, RecoveryInventoryError, RecoveryInventoryLimit,
    RecoveryInventoryOperation, RecoveryInventoryStorage, RecoveryNamespace,
    filesystem_platform_profile, filesystem_recovery_inventory_scan,
    filesystem_recovery_namespace::PinnedRecoveryDirectory, read_recovery_inventory,
};

const STAGING_NAME: &str = "staging";
const SEGMENTS_NAME: &str = "segments";
const CATALOGS_NAME: &str = "catalogs";

/// A pinned, read-only view of the four recovery inventory namespaces.
///
/// Opening performs no protocol mutation. The production constructor admits
/// the same Linux ext4 profile as store initialization and pins all three child
/// directories without following symbolic links.
#[must_use]
pub struct FilesystemRecoveryInventoryReader {
    root: Dir,
    staging: PinnedRecoveryDirectory,
    segments: PinnedRecoveryDirectory,
    catalogs: PinnedRecoveryDirectory,
}

impl FilesystemRecoveryInventoryReader {
    /// Opens one initialized store for read-only recovery inventory.
    ///
    /// The call is synchronous, allocates no content-sized memory, and may
    /// block on filesystem I/O.
    ///
    /// # Errors
    ///
    /// Returns [`RecoveryInventoryError::Io`] with the exact namespace and
    /// open phase for an unsupported platform or missing, replaced, linked, or
    /// non-directory protocol namespace.
    pub fn open(store_root: &Path) -> Result<Self, RecoveryInventoryError> {
        let root = filesystem_platform_profile::open(store_root).map_err(|source| {
            RecoveryInventoryError::io(
                RecoveryNamespace::Root,
                RecoveryInventoryOperation::OpenNamespace,
                source,
            )
        })?;
        Self::from_root(root)
    }

    #[cfg(test)]
    pub(super) fn open_unchecked_for_tests(
        store_root: &Path,
    ) -> Result<Self, RecoveryInventoryError> {
        let root = Dir::open_ambient_dir(store_root, ambient_authority()).map_err(|source| {
            RecoveryInventoryError::io(
                RecoveryNamespace::Root,
                RecoveryInventoryOperation::OpenNamespace,
                source,
            )
        })?;
        Self::from_root(root)
    }

    fn from_root(root: Dir) -> Result<Self, RecoveryInventoryError> {
        let staging =
            PinnedRecoveryDirectory::open(&root, RecoveryNamespace::Staging, STAGING_NAME)?;
        let segments =
            PinnedRecoveryDirectory::open(&root, RecoveryNamespace::Segments, SEGMENTS_NAME)?;
        let catalogs =
            PinnedRecoveryDirectory::open(&root, RecoveryNamespace::Catalogs, CATALOGS_NAME)?;
        Ok(Self {
            root,
            staging,
            segments,
            catalogs,
        })
    }

    /// Reads one bounded, deterministic inventory without mutating protocol
    /// state.
    ///
    /// The call is synchronous and may block on directory I/O. Peak allocation
    /// includes the final `limit`-bounded inventory plus one temporary
    /// `limit + 1`-bounded namespace table used to prove count drift.
    ///
    /// # Errors
    ///
    /// Returns [`RecoveryInventoryError`] on namespace replacement, storage
    /// refusal, entry-limit excess, count drift, or duplicate names.
    pub fn read(
        &mut self,
        limit: RecoveryInventoryLimit,
    ) -> Result<RecoveryInventory, RecoveryInventoryError> {
        self.verify_namespaces()?;
        let inventory = read_recovery_inventory(self, limit)?;
        self.verify_namespaces()?;
        Ok(inventory)
    }

    fn verify_namespaces(&self) -> Result<(), RecoveryInventoryError> {
        self.staging.verify(&self.root)?;
        self.segments.verify(&self.root)?;
        self.catalogs.verify(&self.root)
    }

    const fn directory(&self, namespace: RecoveryNamespace) -> &Dir {
        match namespace {
            RecoveryNamespace::Root => &self.root,
            RecoveryNamespace::Staging => self.staging.directory(),
            RecoveryNamespace::Segments => self.segments.directory(),
            RecoveryNamespace::Catalogs => self.catalogs.directory(),
        }
    }
}

impl RecoveryInventoryStorage for FilesystemRecoveryInventoryReader {
    fn count_entries(&mut self, namespace: RecoveryNamespace, remaining: u64) -> io::Result<u64> {
        filesystem_recovery_inventory_scan::count_entries(self.directory(namespace), remaining)
    }

    fn read_entry_names(
        &mut self,
        namespace: RecoveryNamespace,
        expected_count: u64,
    ) -> io::Result<Vec<RecoveryEntryName>> {
        filesystem_recovery_inventory_scan::read_entry_names(
            self.directory(namespace),
            expected_count,
        )
    }
}
