//! This module owns stage creation and purpose-specific atomic replacement.

use super::{Boundary, Durability, Effect, FilesystemRetentionStage, RetentionStorageError};
use super::{exact_record, retention_error, verify_named_record};
use crate::adapters::filesystem_catalog_artifact;
use crate::adapters::filesystem_exact_record::EntryIdentity;
use cap_std::fs::{Dir, File};
use std::io::{self, Write};

/// The published record replaced by a shared stage capability.
#[derive(Clone, Copy)]
pub(in crate::adapters) enum StageReplacement {
    Head,
    Receipt,
}

impl StageReplacement {
    const fn coordinates(self) -> (Boundary, Boundary, Effect) {
        match self {
            Self::Head => (
                Boundary::HeadRename,
                Boundary::HeadVerification,
                Effect::HeadReplaced,
            ),
            Self::Receipt => (
                Boundary::ReceiptRename,
                Boundary::ReceiptVerification,
                Effect::ReceiptReplaced,
            ),
        }
    }
}

impl FilesystemRetentionStage {
    pub(super) fn create_with(
        root: &Dir,
        name: &'static str,
        expected: &[u8],
        mut before: impl FnMut(Boundary, &mut File) -> io::Result<()>,
    ) -> Result<Self, RetentionStorageError> {
        let mut file =
            filesystem_catalog_artifact::create_exclusive(root, name).map_err(creation_failure)?;
        let identity = before(Boundary::StageIdentity, &mut file)
            .and_then(|()| EntryIdentity::of_file(&file))
            .map_err(|source| created_failure(source, Boundary::StageIdentity))?;
        before(Boundary::StageWrite, &mut file)
            .and_then(|()| file.write_all(expected))
            .map_err(|source| created_failure(source, Boundary::StageWrite))?;
        before(Boundary::StageFlush, &mut file)
            .and_then(|()| file.flush())
            .map_err(|source| created_failure(source, Boundary::StageFlush))?;
        Ok(Self {
            name,
            expected: Box::from(expected),
            identity,
            file,
        })
    }

    pub(super) fn replace_with(
        &self,
        root: &Dir,
        name: &str,
        purpose: StageReplacement,
        after_rename: impl FnOnce() -> io::Result<()>,
    ) -> Result<(), RetentionStorageError> {
        self.verify_stage(root)
            .map_err(|error| error.at(Boundary::SourceVerification))?;
        let (rename, verification, effect) = purpose.coordinates();
        root.rename(self.name, root, name).map_err(|source| {
            RetentionStorageError::from(source)
                .at(rename)
                .uncertain(effect)
        })?;
        after_rename().map_err(|source| {
            RetentionStorageError::from(source)
                .at(verification)
                .after(effect, Durability::Unconfirmed)
        })?;
        exact_record::require_absent(root, self.name).map_err(|error| {
            retention_error(error)
                .at(Boundary::StageAbsence)
                .after(effect, Durability::Unconfirmed)
        })?;
        verify_named_record(root, name, &self.expected, self.identity).map_err(|error| {
            error
                .at(verification)
                .after(effect, Durability::Unconfirmed)
        })
    }
}

fn creation_failure(source: io::Error) -> RetentionStorageError {
    let existed = source.kind() == io::ErrorKind::AlreadyExists;
    let error = RetentionStorageError::from(source).at(Boundary::StageCreation);
    if existed {
        error
    } else {
        error.uncertain(Effect::StageCreated)
    }
}

fn created_failure(source: io::Error, boundary: Boundary) -> RetentionStorageError {
    RetentionStorageError::from(source)
        .at(boundary)
        .after(Effect::StageCreated, Durability::Unconfirmed)
}
