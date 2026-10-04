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

impl FilesystemRetentionStage {
    pub(super) fn create_with(
        root: &Dir,
        name: &'static str,
        expected: &[u8],
        mut before: impl FnMut(Boundary, &mut File) -> io::Result<()>,
    ) -> Result<Self, RetentionStorageError> {
        let mut file = filesystem_catalog_artifact::create_exclusive(root, name)?;
        before(Boundary::StageIdentity, &mut file)?;
        let identity = EntryIdentity::of_file(&file)?;
        before(Boundary::StageWrite, &mut file)?;
        file.write_all(expected)?;
        before(Boundary::StageFlush, &mut file)?;
        file.flush()?;
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
        _purpose: StageReplacement,
        after_rename: impl FnOnce() -> io::Result<()>,
    ) -> Result<(), RetentionStorageError> {
        self.verify_stage(root)
            .map_err(|error| error.at(Boundary::SourceVerification))?;
        root.rename(self.name, root, name).map_err(|source| {
            RetentionStorageError::from(source)
                .at(Boundary::HeadRename)
                .uncertain(Effect::HeadReplaced)
        })?;
        after_rename()?;
        exact_record::require_absent(root, self.name).map_err(|error| {
            retention_error(error)
                .at(Boundary::StageAbsence)
                .after(Effect::HeadReplaced, Durability::Unconfirmed)
        })?;
        verify_named_record(root, name, &self.expected, self.identity).map_err(|error| {
            error
                .at(Boundary::HeadVerification)
                .after(Effect::HeadReplaced, Durability::Unconfirmed)
        })
    }
}
