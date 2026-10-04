//! This module owns exact variable-length retention stage publication.

use super::{
    RetentionEffectDurability as Durability, RetentionNamespaceEffect as Effect,
    RetentionStorageBoundary as Boundary,
};
use super::{RetentionRecordRefusal, RetentionStorageError};
use std::io;

use cap_std::fs::{Dir, File};

#[path = "filesystem_retention_stage_io.rs"]
mod stage_io;
pub(in crate::adapters) use stage_io::StageReplacement;
#[cfg(test)]
#[path = "filesystem_retention_stage_effect_tests.rs"]
mod effect_tests;
use crate::adapters::filesystem_exact_record::{
    self as exact_record, EntryIdentity, ExactRecordError, ExactRecordRefusal,
};

/// Whether a no-replacement link call created a namespace entry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::adapters) enum RetentionLinkOutcome {
    Created,
    Existing,
}

impl RetentionLinkOutcome {
    pub(in crate::adapters) fn report(self, error: RetentionStorageError) -> RetentionStorageError {
        match self {
            Self::Created => error.after(Effect::PoolLinkCreated, Durability::Unconfirmed),
            Self::Existing => error,
        }
    }
}

/// One exclusively created, verified, and retained retention stage file.
///
/// The stage retains its opened handle and recorded device and inode identity
/// for its whole lifetime. Every transition reverifies both the handle and the
/// named entry, so a replaced or byte-equal substituted file refuses instead of
/// being admitted.
pub(in crate::adapters) struct FilesystemRetentionStage {
    name: &'static str,
    expected: Box<[u8]>,
    identity: EntryIdentity,
    file: File,
}

impl FilesystemRetentionStage {
    /// Exclusively creates the named stage and writes its complete bytes.
    ///
    /// Failure retains any created stage and its possibly partial contents.
    /// Reported creation is a namespace effect, not proof of complete bytes or
    /// directory durability; the original operational cause is preserved.
    pub(in crate::adapters) fn create(
        root: &Dir,
        name: &'static str,
        expected: &[u8],
    ) -> Result<Self, RetentionStorageError> {
        Self::create_with(root, name, expected, |_, _| Ok(()))
    }

    /// Reopens a retained stage whose exact bytes restart already read.
    ///
    /// The handle and the named entry are verified against `expected` and
    /// bound to the observed entry's identity, so every later transition refuses a
    /// substituted or replaced stage exactly as a freshly created one would.
    pub(in crate::adapters) fn reopen(
        root: &Dir,
        name: &'static str,
        expected: &[u8],
        identity: EntryIdentity,
    ) -> Result<Self, RetentionStorageError> {
        let file = exact_record::open_read(root, name)?;
        verify_named_record(root, name, expected, identity)?;
        Ok(Self {
            name,
            expected: Box::from(expected),
            identity,
            file,
        })
    }

    /// Synchronizes the complete stage and reverifies its exact bytes.
    pub(in crate::adapters) fn synchronize(&self, root: &Dir) -> Result<(), RetentionStorageError> {
        self.verify_stage(root)
            .map_err(|error| error.at(Boundary::SourceVerification))?;
        self.file.sync_all().map_err(|source| {
            RetentionStorageError::from(source).at(Boundary::StageSynchronization)
        })?;
        self.verify_stage(root)
            .map_err(|error| error.at(Boundary::SourceVerification))
    }

    /// Links the verified stage into `target` under `name` without replacement.
    pub(in crate::adapters) fn link(
        &self,
        root: &Dir,
        target: &Dir,
        name: &str,
    ) -> Result<RetentionLinkOutcome, RetentionStorageError> {
        self.verify_stage(root)
            .map_err(|error| error.at(Boundary::SourceVerification))?;
        let outcome = match root.hard_link(self.name, target, name) {
            Ok(()) => RetentionLinkOutcome::Created,
            Err(source) if source.kind() == io::ErrorKind::AlreadyExists => {
                RetentionLinkOutcome::Existing
            }
            Err(source) => {
                return Err(RetentionStorageError::from(source)
                    .at(Boundary::PoolLink)
                    .uncertain(Effect::PoolLinkCreated));
            }
        };
        self.verify_stage(root)
            .map_err(|error| outcome.report(error.at(Boundary::SourceVerification)))?;
        verify_named_record(target, name, &self.expected, self.identity)
            .map_err(|error| outcome.report(error.at(Boundary::PoolVerification)))?;
        Ok(outcome)
    }

    /// Removes only the retained stage after confirming its linked target.
    pub(in crate::adapters) fn remove(
        &self,
        root: &Dir,
        target: &Dir,
        name: &str,
    ) -> Result<(), RetentionStorageError> {
        self.verify_stage(root)
            .map_err(|error| error.at(Boundary::SourceVerification))?;
        verify_named_record(target, name, &self.expected, self.identity)
            .map_err(|error| error.at(Boundary::PoolVerification))?;
        root.remove_file(self.name).map_err(|source| {
            RetentionStorageError::from(source)
                .at(Boundary::StageUnlink)
                .uncertain(Effect::StageRemoved)
        })?;
        exact_record::require_absent(root, self.name).map_err(|error| {
            retention_error(error)
                .at(Boundary::StageAbsence)
                .after(Effect::StageRemoved, Durability::Unconfirmed)
        })?;
        verify_named_record(target, name, &self.expected, self.identity).map_err(|error| {
            error
                .at(Boundary::PoolVerification)
                .after(Effect::StageRemoved, Durability::Unconfirmed)
        })
    }

    /// Renames the verified stage onto `name`, replacing it atomically.
    ///
    /// The purpose identifies head versus receipt effects. Failure after rename
    /// does not restore the source pathname or establish directory durability.
    pub(in crate::adapters) fn replace(
        &self,
        root: &Dir,
        name: &str,
        purpose: StageReplacement,
    ) -> Result<(), RetentionStorageError> {
        self.replace_with(root, name, purpose, || Ok(()))
    }

    fn require_handle(&self) -> Result<(), RetentionStorageError> {
        if EntryIdentity::of_file(&self.file)? == self.identity {
            Ok(())
        } else {
            Err(RetentionStorageError::Refused {
                source: RetentionRecordRefusal::KindLengthOrIdentity,
            })
        }
    }

    fn verify_stage(&self, root: &Dir) -> Result<(), RetentionStorageError> {
        self.require_handle()?;
        verify_named_record(root, self.name, &self.expected, self.identity)
    }
}

fn verify_named_record(
    directory: &Dir,
    name: &str,
    expected: &[u8],
    identity: EntryIdentity,
) -> Result<(), RetentionStorageError> {
    exact_record::verify_named(directory, name, expected, identity).map_err(retention_error)
}

/// Adapts exact-record failures without erasing their semantic distinctions.
pub(in crate::adapters) fn retention_error(error: ExactRecordError) -> RetentionStorageError {
    match error {
        ExactRecordError::Io(source) => RetentionStorageError::Io { source },
        ExactRecordError::Refused(refusal) => RetentionStorageError::Refused {
            source: match refusal {
                ExactRecordRefusal::LengthOverflow => RetentionRecordRefusal::LengthOverflow,
                ExactRecordRefusal::KindOrLength => RetentionRecordRefusal::KindOrLength,
                ExactRecordRefusal::KindLengthOrIdentity => {
                    RetentionRecordRefusal::KindLengthOrIdentity
                }
                ExactRecordRefusal::Bytes => RetentionRecordRefusal::Bytes,
                ExactRecordRefusal::TrailingBytes => RetentionRecordRefusal::TrailingBytes,
                ExactRecordRefusal::RemainedVisible => RetentionRecordRefusal::RemainedVisible,
            },
        },
    }
}

pub(in crate::adapters) fn invalid_data(
    refusal: super::FilesystemRetentionStageRefusal,
) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, refusal)
}
