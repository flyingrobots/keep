//! This module owns identity-stable migration artifact reads.

use cap_fs_ext::MetadataExt;
use cap_std::fs::{Dir, File, Metadata};

use crate::adapters::{
    CatalogRestartArtifact, CatalogRestartError, CatalogRestartPhase, catalog_restart_io,
};

pub(super) enum FilesystemInventoryFileError {
    Artifact(Box<CatalogRestartError>),
    Changed,
}

#[derive(Clone, Copy)]
pub(super) struct FilesystemInventoryFilePolicy {
    artifact: CatalogRestartArtifact,
    open_phase: CatalogRestartPhase,
    read_phase: CatalogRestartPhase,
    maximum_length: u64,
}

impl FilesystemInventoryFilePolicy {
    pub(super) const fn new(
        artifact: CatalogRestartArtifact,
        open_phase: CatalogRestartPhase,
        read_phase: CatalogRestartPhase,
        maximum_length: u64,
    ) -> Self {
        Self {
            artifact,
            open_phase,
            read_phase,
            maximum_length,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct FileIdentity {
    device: u64,
    inode: u64,
    length: u64,
}

impl FileIdentity {
    fn read(file: &File, phase: CatalogRestartPhase) -> Result<Self, CatalogRestartError> {
        file.metadata()
            .map(|metadata| Self::from(&metadata))
            .map_err(|source| CatalogRestartError::io(phase, source))
    }
}

impl From<&Metadata> for FileIdentity {
    fn from(metadata: &Metadata) -> Self {
        Self {
            device: metadata.dev(),
            inode: metadata.ino(),
            length: metadata.len(),
        }
    }
}

pub(super) fn read(
    directory: &Dir,
    name: &str,
    policy: FilesystemInventoryFilePolicy,
) -> Result<Vec<u8>, FilesystemInventoryFileError> {
    read_with(directory, name, policy, || {})
}

pub(super) fn read_with<F>(
    directory: &Dir,
    name: &str,
    policy: FilesystemInventoryFilePolicy,
    before_verify: F,
) -> Result<Vec<u8>, FilesystemInventoryFileError>
where
    F: FnOnce(),
{
    let (file, length) =
        catalog_restart_io::open_regular(directory, name, policy.artifact, policy.open_phase)
            .map_err(artifact_error)?;
    if length > policy.maximum_length {
        return Err(FilesystemInventoryFileError::Artifact(Box::new(
            CatalogRestartError::Length {
                artifact: policy.artifact,
                minimum: 0,
                maximum: policy.maximum_length,
                observed: length,
            },
        )));
    }
    let identity = FileIdentity::read(&file, policy.read_phase).map_err(artifact_error)?;
    let retained = file
        .try_clone()
        .map_err(|source| CatalogRestartError::io(policy.read_phase, source))
        .map_err(artifact_error)?;
    let encoded = catalog_restart_io::read_exact(file, policy.artifact, policy.read_phase, length)
        .map_err(artifact_error)?;
    before_verify();
    verify(directory, name, &retained, identity, policy.read_phase)?;
    Ok(encoded)
}

fn verify(
    directory: &Dir,
    name: &str,
    file: &File,
    identity: FileIdentity,
    phase: CatalogRestartPhase,
) -> Result<(), FilesystemInventoryFileError> {
    let handle = FileIdentity::read(file, phase).map_err(artifact_error)?;
    let metadata = directory
        .symlink_metadata(name)
        .map_err(|source| CatalogRestartError::io(phase, source))
        .map_err(artifact_error)?;
    let current = FileIdentity::from(&metadata);
    if metadata.is_file() && handle == identity && current == identity {
        Ok(())
    } else {
        Err(FilesystemInventoryFileError::Changed)
    }
}

fn artifact_error(source: CatalogRestartError) -> FilesystemInventoryFileError {
    FilesystemInventoryFileError::Artifact(Box::new(source))
}
