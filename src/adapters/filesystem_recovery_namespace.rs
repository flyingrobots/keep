//! This module owns pinned recovery namespace identity.

use std::io;

use cap_std::fs::Dir;

use super::filesystem_exact_record::EntryIdentity;
use super::{FilesystemEntryKind, FilesystemOperationRefusal};

use super::{
    RecoveryInventoryError, RecoveryInventoryOperation, RecoveryNamespace, sync_capable_directory,
};

pub(super) struct PinnedRecoveryDirectory {
    namespace: RecoveryNamespace,
    name: &'static str,
    identity: EntryIdentity,
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
        let identity = EntryIdentity::of_directory(&directory).map_err(|source| {
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
        let observed = EntryIdentity::from(&metadata);
        let refusal = if !metadata.is_dir() {
            FilesystemOperationRefusal::DirectoryRequired {
                observed: FilesystemEntryKind::observed(metadata.file_type()),
            }
        } else if observed == self.identity {
            return Ok(());
        } else {
            FilesystemOperationRefusal::IdentityChanged {
                expected: self.identity.into(),
                observed: observed.into(),
            }
        };
        Err(RecoveryInventoryError::io(
            self.namespace,
            RecoveryInventoryOperation::VerifyNamespace,
            io::Error::new(io::ErrorKind::InvalidData, refusal),
        ))
    }

    pub(super) const fn directory(&self) -> &Dir {
        &self.directory
    }
}
