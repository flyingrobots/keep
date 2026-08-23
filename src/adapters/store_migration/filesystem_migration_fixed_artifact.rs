//! This module owns exact fixed-record migration publication.

use std::io::{self, Read, Write};

use cap_fs_ext::{FollowSymlinks, MetadataExt, OpenOptionsFollowExt, OpenOptionsSyncExt};
use cap_std::fs::{Dir, File, Metadata, OpenOptions};

use super::{format_marker_decoder, migration_intent_format, migration_receipt_format};
use crate::adapters::filesystem_catalog_artifact;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum FilesystemMigrationFixedArtifact {
    Intent,
    Marker,
    Receipt,
}

impl FilesystemMigrationFixedArtifact {
    const fn stage_name(self) -> &'static str {
        match self {
            Self::Intent => "migration.intent.next",
            Self::Marker => "FORMAT.next",
            Self::Receipt => "migration.receipt.next",
        }
    }

    const fn canonical_name(self) -> &'static str {
        match self {
            Self::Intent => "migration.intent",
            Self::Marker => "FORMAT",
            Self::Receipt => "migration.receipt",
        }
    }

    const fn encoded_length(self) -> usize {
        match self {
            Self::Intent => migration_intent_format::ENCODED_LENGTH,
            Self::Marker => format_marker_decoder::ENCODED_LENGTH,
            Self::Receipt => migration_receipt_format::ENCODED_LENGTH,
        }
    }
}

pub(super) struct FilesystemMigrationFixedStage {
    artifact: FilesystemMigrationFixedArtifact,
    expected: Box<[u8]>,
    identity: FixedFileIdentity,
    file: File,
}

impl FilesystemMigrationFixedStage {
    pub(super) fn create(
        root: &Dir,
        artifact: FilesystemMigrationFixedArtifact,
        expected: &[u8],
    ) -> io::Result<Self> {
        require_length(artifact, expected)?;
        let mut file = filesystem_catalog_artifact::create_exclusive(root, artifact.stage_name())?;
        let identity = FixedFileIdentity::read_file(&file)?;
        file.write_all(expected)?;
        file.flush()?;
        Ok(Self {
            artifact,
            expected: Box::from(expected),
            identity,
            file,
        })
    }

    pub(super) fn synchronize(&self, root: &Dir) -> io::Result<()> {
        self.require_handle()?;
        self.file.sync_all()?;
        self.verify_stage(root)
    }

    pub(super) fn link(
        &self,
        root: &Dir,
        artifact: FilesystemMigrationFixedArtifact,
        expected: &[u8],
    ) -> io::Result<()> {
        self.require_record(artifact, expected)?;
        self.verify_stage(root)?;
        match root.hard_link(
            self.artifact.stage_name(),
            root,
            self.artifact.canonical_name(),
        ) {
            Ok(()) => {}
            Err(source) if source.kind() == io::ErrorKind::AlreadyExists => {}
            Err(source) => return Err(source),
        }
        self.verify_linked_names(root)
    }

    pub(super) fn remove(self, root: &Dir) -> io::Result<Self> {
        self.verify_linked_names(root)?;
        root.remove_file(self.artifact.stage_name())?;
        require_absent(root, self.artifact.stage_name())?;
        self.verify_canonical(root)?;
        Ok(self)
    }

    pub(super) fn verify_linked(&self, root: &Dir) -> io::Result<()> {
        self.verify_linked_names(root)
    }

    pub(super) fn verify_canonical(&self, root: &Dir) -> io::Result<()> {
        self.require_handle()?;
        verify_name(
            root,
            self.artifact.canonical_name(),
            &self.expected,
            self.identity,
        )
    }

    pub(super) const fn artifact(&self) -> FilesystemMigrationFixedArtifact {
        self.artifact
    }

    fn require_handle(&self) -> io::Result<()> {
        let observed = FixedFileIdentity::read_file(&self.file)?;
        if observed == self.identity {
            Ok(())
        } else {
            Err(invalid_data("migration stage handle changed identity"))
        }
    }

    fn require_record(
        &self,
        artifact: FilesystemMigrationFixedArtifact,
        expected: &[u8],
    ) -> io::Result<()> {
        if self.artifact == artifact && self.expected.as_ref() == expected {
            Ok(())
        } else {
            Err(invalid_data("migration stage record disagreed"))
        }
    }

    fn verify_stage(&self, root: &Dir) -> io::Result<()> {
        verify_name(
            root,
            self.artifact.stage_name(),
            &self.expected,
            self.identity,
        )
    }

    fn verify_linked_names(&self, root: &Dir) -> io::Result<()> {
        self.verify_stage(root)?;
        verify_name(
            root,
            self.artifact.canonical_name(),
            &self.expected,
            self.identity,
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct FixedFileIdentity {
    device: u64,
    inode: u64,
}

impl FixedFileIdentity {
    fn read_file(file: &File) -> io::Result<Self> {
        file.metadata().map(|metadata| Self::from(&metadata))
    }
}

impl From<&Metadata> for FixedFileIdentity {
    fn from(metadata: &Metadata) -> Self {
        Self {
            device: metadata.dev(),
            inode: metadata.ino(),
        }
    }
}

fn verify_name(
    root: &Dir,
    name: &str,
    expected: &[u8],
    identity: FixedFileIdentity,
) -> io::Result<()> {
    let mut options = OpenOptions::new();
    options.read(true).follow(FollowSymlinks::No).nonblock(true);
    let mut file = root.open_with(name, &options)?;
    require_metadata(&file.metadata()?, expected.len(), identity)?;
    require_metadata(&root.symlink_metadata(name)?, expected.len(), identity)?;
    let mut observed = vec![0_u8; expected.len()];
    file.read_exact(&mut observed)?;
    let mut trailing = [0_u8; 1];
    if observed != expected || file.read(&mut trailing)? != 0 {
        return Err(invalid_data("migration fixed-record bytes disagreed"));
    }
    require_metadata(&file.metadata()?, expected.len(), identity)?;
    require_metadata(&root.symlink_metadata(name)?, expected.len(), identity)
}

fn require_metadata(
    metadata: &Metadata,
    expected_length: usize,
    expected_identity: FixedFileIdentity,
) -> io::Result<()> {
    let expected_length = u64::try_from(expected_length)
        .map_err(|_source| invalid_data("migration fixed-record length exceeded u64"))?;
    if metadata.is_file()
        && metadata.len() == expected_length
        && FixedFileIdentity::from(metadata) == expected_identity
    {
        Ok(())
    } else {
        Err(invalid_data(
            "migration fixed-record kind, length, or identity disagreed",
        ))
    }
}

fn require_length(artifact: FilesystemMigrationFixedArtifact, expected: &[u8]) -> io::Result<()> {
    if expected.len() == artifact.encoded_length() {
        Ok(())
    } else {
        Err(invalid_data("migration fixed-record length disagreed"))
    }
}

fn require_absent(root: &Dir, name: &str) -> io::Result<()> {
    match root.symlink_metadata(name) {
        Err(source) if source.kind() == io::ErrorKind::NotFound => Ok(()),
        Ok(_) => Err(invalid_data("removed migration stage remained visible")),
        Err(source) => Err(source),
    }
}

fn invalid_data(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
