//! This module owns exact fixed-record migration publication.

use std::io::{self, Write};

use cap_std::fs::{Dir, File};

use super::{format_marker_decoder, migration_intent_format, migration_receipt_format};
use crate::adapters::filesystem_catalog_artifact;
use crate::adapters::filesystem_exact_record::{
    self as exact_record, EntryIdentity, ExactRecordError, ExactRecordRefusal,
};

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
    identity: EntryIdentity,
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
        let identity = EntryIdentity::of_file(&file)?;
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
        exact_record::link_without_replacement(
            root,
            self.artifact.stage_name(),
            root,
            self.artifact.canonical_name(),
        )?;
        self.verify_linked_names(root)
    }

    pub(super) fn remove(self, root: &Dir) -> io::Result<Self> {
        self.verify_linked_names(root)?;
        root.remove_file(self.artifact.stage_name())?;
        exact_record::require_absent(root, self.artifact.stage_name()).map_err(migration_error)?;
        self.verify_canonical(root)?;
        Ok(self)
    }

    pub(super) fn verify_linked(&self, root: &Dir) -> io::Result<()> {
        self.verify_linked_names(root)
    }

    pub(super) fn verify_canonical(&self, root: &Dir) -> io::Result<()> {
        self.require_handle()?;
        verify_named_record(
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
        let observed = EntryIdentity::of_file(&self.file)?;
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
        verify_named_record(
            root,
            self.artifact.stage_name(),
            &self.expected,
            self.identity,
        )
    }

    fn verify_linked_names(&self, root: &Dir) -> io::Result<()> {
        self.verify_stage(root)?;
        verify_named_record(
            root,
            self.artifact.canonical_name(),
            &self.expected,
            self.identity,
        )
    }
}

fn verify_named_record(
    root: &Dir,
    name: &str,
    expected: &[u8],
    identity: EntryIdentity,
) -> io::Result<()> {
    exact_record::verify_named(root, name, expected, identity).map_err(migration_error)
}

/// Maps a shared exact-record failure onto this protocol's refusal messages.
fn migration_error(error: ExactRecordError) -> io::Error {
    match error {
        ExactRecordError::Io(source) => source,
        ExactRecordError::Refused(refusal) => invalid_data(match refusal {
            ExactRecordRefusal::LengthOverflow => "migration fixed-record length exceeded u64",
            ExactRecordRefusal::KindLengthOrIdentity => {
                "migration fixed-record kind, length, or identity disagreed"
            }
            ExactRecordRefusal::Bytes | ExactRecordRefusal::TrailingBytes => {
                "migration fixed-record bytes disagreed"
            }
            ExactRecordRefusal::RemainedVisible => "removed migration stage remained visible",
        }),
    }
}

fn require_length(artifact: FilesystemMigrationFixedArtifact, expected: &[u8]) -> io::Result<()> {
    if expected.len() == artifact.encoded_length() {
        Ok(())
    } else {
        Err(invalid_data("migration fixed-record length disagreed"))
    }
}

fn invalid_data(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
