//! This module owns pinned migration pool-directory identity.

use cap_fs_ext::MetadataExt;
use cap_std::fs::{Dir, Metadata};

use super::filesystem_inventory_error::{
    FilesystemMigrationInventoryError, FilesystemMigrationInventoryOperation,
    MigrationInventoryNamespace, MigrationInventoryPool,
};
use crate::adapters::sync_capable_directory;

pub(super) struct PinnedMigrationPoolDirectory {
    pool: MigrationInventoryPool,
    name: &'static str,
    identity: DirectoryIdentity,
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
        let identity = DirectoryIdentity::read(&directory).map_err(|source| {
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
        let handle = DirectoryIdentity::read(&self.directory).map_err(|source| {
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
        let current = DirectoryIdentity::from(&metadata);
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct DirectoryIdentity {
    device: u64,
    inode: u64,
}

impl DirectoryIdentity {
    fn read(directory: &Dir) -> std::io::Result<Self> {
        directory
            .dir_metadata()
            .map(|metadata| Self::from(&metadata))
    }
}

impl From<&Metadata> for DirectoryIdentity {
    fn from(metadata: &Metadata) -> Self {
        Self {
            device: metadata.dev(),
            inode: metadata.ino(),
        }
    }
}
