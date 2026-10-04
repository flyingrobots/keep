//! This module owns pinned migration pool-directory identity.

use cap_std::fs::Dir;

use crate::adapters::filesystem_exact_record::EntryIdentity;

use super::filesystem_inventory_error::{
    FilesystemMigrationInventoryError, FilesystemMigrationInventoryOperation,
    MigrationInventoryNamespace, MigrationInventoryPool,
};
use crate::adapters::sync_capable_directory;

pub(super) struct PinnedMigrationPoolDirectory {
    pool: MigrationInventoryPool,
    name: &'static str,
    identity: EntryIdentity,
    directory: Dir,
}

impl PinnedMigrationPoolDirectory {
    pub(super) fn open(
        root: &Dir,
        pool: MigrationInventoryPool,
        name: &'static str,
    ) -> Result<Self, FilesystemMigrationInventoryError> {
        let directory = sync_capable_directory::open(root, name).map_err(|source| {
            FilesystemMigrationInventoryError::Io {
                namespace: MigrationInventoryNamespace::from(pool),
                operation: FilesystemMigrationInventoryOperation::OpenPool,
                source,
            }
        })?;
        let identity = EntryIdentity::of_directory(&directory).map_err(|source| {
            FilesystemMigrationInventoryError::Io {
                namespace: MigrationInventoryNamespace::from(pool),
                operation: FilesystemMigrationInventoryOperation::OpenPool,
                source,
            }
        })?;
        Ok(Self {
            pool,
            name,
            identity,
            directory,
        })
    }

    pub(super) fn verify(&self, root: &Dir) -> Result<(), FilesystemMigrationInventoryError> {
        let handle = EntryIdentity::of_directory(&self.directory).map_err(|source| {
            FilesystemMigrationInventoryError::Io {
                namespace: MigrationInventoryNamespace::from(self.pool),
                operation: FilesystemMigrationInventoryOperation::VerifyPool,
                source,
            }
        })?;
        let metadata = root.symlink_metadata(self.name).map_err(|source| {
            FilesystemMigrationInventoryError::Io {
                namespace: MigrationInventoryNamespace::from(self.pool),
                operation: FilesystemMigrationInventoryOperation::VerifyPool,
                source,
            }
        })?;
        let current = EntryIdentity::from(&metadata);
        if metadata.is_dir() && handle == self.identity && current == self.identity {
            Ok(())
        } else {
            Err(FilesystemMigrationInventoryError::NamespaceChanged { pool: self.pool })
        }
    }

    pub(super) const fn directory(&self) -> &Dir {
        &self.directory
    }
}
