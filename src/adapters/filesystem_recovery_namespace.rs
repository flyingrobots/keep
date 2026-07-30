//! This module owns pinned recovery namespace identity.

use std::io;

use cap_fs_ext::MetadataExt;
use cap_std::fs::{Dir, Metadata};

use super::{
    RecoveryInventoryError, RecoveryInventoryOperation, RecoveryNamespace, sync_capable_directory,
};

pub(super) struct PinnedRecoveryDirectory {
    namespace: RecoveryNamespace,
    name: &'static str,
    identity: DirectoryIdentity,
    directory: Dir,
}

impl PinnedRecoveryDirectory {
    pub(super) fn open(
        root: &Dir,
        namespace: RecoveryNamespace,
        name: &'static str,
    ) -> Result<Self, RecoveryInventoryError> {
        let directory = sync_capable_directory::open(root, name).map_err(|source| {
            RecoveryInventoryError::io(namespace, RecoveryInventoryOperation::OpenNamespace, source)
        })?;
        let identity = DirectoryIdentity::read(&directory).map_err(|source| {
            RecoveryInventoryError::io(namespace, RecoveryInventoryOperation::OpenNamespace, source)
        })?;
        Ok(Self {
            namespace,
            name,
            identity,
            directory,
        })
    }

    pub(super) fn verify(&self, root: &Dir) -> Result<(), RecoveryInventoryError> {
        let metadata = root.symlink_metadata(self.name).map_err(|source| {
            RecoveryInventoryError::io(
                self.namespace,
                RecoveryInventoryOperation::VerifyNamespace,
                source,
            )
        })?;
        let observed = DirectoryIdentity::from(&metadata);
        if metadata.is_dir() && observed == self.identity {
            return Ok(());
        }
        Err(RecoveryInventoryError::io(
            self.namespace,
            RecoveryInventoryOperation::VerifyNamespace,
            io::Error::new(
                io::ErrorKind::InvalidData,
                "recovery namespace changed identity after it was pinned",
            ),
        ))
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
    fn read(directory: &Dir) -> io::Result<Self> {
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
