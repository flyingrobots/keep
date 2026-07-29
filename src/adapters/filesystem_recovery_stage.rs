//! This module owns pinned filesystem recovery-stage observation.

use std::io::{Seek, SeekFrom};

use cap_fs_ext::{FollowSymlinks, MetadataExt, OpenOptionsFollowExt, OpenOptionsSyncExt};
use cap_std::fs::{Dir, File, Metadata, OpenOptions};

use super::{
    FilesystemRecoveryStageError, RecoveryStage, RecoveryStageEvidence, RecoveryStageLength,
    RecoveryStageMetadata, filesystem_recovery_stage_materialization, fingerprint_recovery_stage,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct FileIdentity {
    device: u64,
    inode: u64,
}

impl From<&Metadata> for FileIdentity {
    fn from(metadata: &Metadata) -> Self {
        Self {
            device: metadata.dev(),
            inode: metadata.ino(),
        }
    }
}

struct AdmittedStage {
    identity: FileIdentity,
    metadata: RecoveryStageMetadata,
}

pub(super) struct ObservedRecoveryStage {
    file: File,
    admitted: AdmittedStage,
    evidence: RecoveryStageEvidence,
}

impl ObservedRecoveryStage {
    pub(super) const fn evidence(&self) -> RecoveryStageEvidence {
        self.evidence
    }

    pub(super) fn synchronize(
        &self,
        directory: &Dir,
        name: &str,
        stage: RecoveryStage,
    ) -> Result<(), FilesystemRecoveryStageError> {
        self.file
            .sync_all()
            .map_err(|source| FilesystemRecoveryStageError::Synchronize { stage, source })?;
        verify_opened_handle(&self.file, stage, &self.admitted)?;
        verify_current_entry(directory, name, stage, &self.admitted)
    }

    pub(super) fn materialize_and_position(
        &mut self,
        stage: RecoveryStage,
    ) -> Result<Box<[u8]>, FilesystemRecoveryStageError> {
        let length = self.admitted.metadata.length();
        filesystem_recovery_stage_materialization::read_and_position(&mut self.file, stage, length)
    }

    pub(super) fn verify(
        &self,
        directory: &Dir,
        name: &str,
        stage: RecoveryStage,
    ) -> Result<(), FilesystemRecoveryStageError> {
        verify_opened_handle(&self.file, stage, &self.admitted)?;
        verify_current_entry(directory, name, stage, &self.admitted)
    }

    pub(super) fn refingerprint_and_position(
        &mut self,
        stage: RecoveryStage,
    ) -> Result<RecoveryStageEvidence, FilesystemRecoveryStageError> {
        let length = self.admitted.metadata.length();
        self.file
            .seek(SeekFrom::Start(0))
            .map_err(|source| FilesystemRecoveryStageError::Position { stage, source })?;
        let evidence = fingerprint_recovery_stage(self.admitted.metadata, &mut self.file)
            .map_err(|source| FilesystemRecoveryStageError::Fingerprint { stage, source })?;
        verify_length(stage, length, evidence.length().get())?;
        filesystem_recovery_stage_materialization::verify_position(&mut self.file, stage, length)?;
        Ok(evidence)
    }

    pub(super) fn into_file(self) -> File {
        self.file
    }
}

pub(super) fn fingerprint(
    directory: &Dir,
    stage: RecoveryStage,
) -> Result<RecoveryStageEvidence, FilesystemRecoveryStageError> {
    Ok(observe(directory, stage)?.evidence())
}

pub(super) fn observe(
    directory: &Dir,
    stage: RecoveryStage,
) -> Result<ObservedRecoveryStage, FilesystemRecoveryStageError> {
    observe_named(directory, stage.file_name(), stage)
}

pub(super) fn fingerprint_named(
    directory: &Dir,
    name: &str,
    stage: RecoveryStage,
) -> Result<RecoveryStageEvidence, FilesystemRecoveryStageError> {
    Ok(observe_named(directory, name, stage)?.evidence())
}

#[cfg(test)]
pub(super) fn fingerprint_with<F>(
    directory: &Dir,
    stage: RecoveryStage,
    after_open: F,
) -> Result<RecoveryStageEvidence, FilesystemRecoveryStageError>
where
    F: FnOnce(),
{
    Ok(observe_named_with(directory, stage.file_name(), stage, after_open)?.evidence())
}

pub(super) fn observe_named(
    directory: &Dir,
    name: &str,
    stage: RecoveryStage,
) -> Result<ObservedRecoveryStage, FilesystemRecoveryStageError> {
    observe_named_with(directory, name, stage, || {})
}

pub(super) fn observe_named_with<F>(
    directory: &Dir,
    name: &str,
    stage: RecoveryStage,
    after_open: F,
) -> Result<ObservedRecoveryStage, FilesystemRecoveryStageError>
where
    F: FnOnce(),
{
    observe_named_with_options(directory, name, stage, &read_options(), after_open)
}

pub(super) fn observe_writable_segment_with<F>(
    directory: &Dir,
    after_open: F,
) -> Result<ObservedRecoveryStage, FilesystemRecoveryStageError>
where
    F: FnOnce(),
{
    observe_named_with_options(
        directory,
        RecoveryStage::Segment.file_name(),
        RecoveryStage::Segment,
        &read_write_options(),
        after_open,
    )
}

fn observe_named_with_options<F>(
    directory: &Dir,
    name: &str,
    stage: RecoveryStage,
    options: &OpenOptions,
    after_open: F,
) -> Result<ObservedRecoveryStage, FilesystemRecoveryStageError>
where
    F: FnOnce(),
{
    let mut file = open_stage(directory, name, stage, options)?;
    let admitted = admit_stage(&file, stage)?;
    after_open();
    let evidence = fingerprint_recovery_stage(admitted.metadata, &mut file)
        .map_err(|source| FilesystemRecoveryStageError::Fingerprint { stage, source })?;
    verify_length(stage, admitted.metadata.length(), evidence.length().get())?;
    verify_opened_handle(&file, stage, &admitted)?;
    verify_current_entry(directory, name, stage, &admitted)?;
    Ok(ObservedRecoveryStage {
        file,
        admitted,
        evidence,
    })
}

fn open_stage(
    directory: &Dir,
    name: &str,
    stage: RecoveryStage,
    options: &OpenOptions,
) -> Result<File, FilesystemRecoveryStageError> {
    directory
        .open_with(name, options)
        .map_err(|source| FilesystemRecoveryStageError::Open { stage, source })
}

fn admit_stage(
    file: &File,
    stage: RecoveryStage,
) -> Result<AdmittedStage, FilesystemRecoveryStageError> {
    let metadata = file
        .metadata()
        .map_err(|source| FilesystemRecoveryStageError::Inspect { stage, source })?;
    if !metadata.is_file() {
        return Err(FilesystemRecoveryStageError::NonRegular { stage });
    }
    let identity = FileIdentity::from(&metadata);
    let metadata = RecoveryStageMetadata::new(stage, metadata.len())
        .map_err(|source| FilesystemRecoveryStageError::MetadataAdmission { stage, source })?;
    Ok(AdmittedStage { identity, metadata })
}

fn verify_opened_handle(
    file: &File,
    stage: RecoveryStage,
    admitted: &AdmittedStage,
) -> Result<(), FilesystemRecoveryStageError> {
    let metadata = file
        .metadata()
        .map_err(|source| FilesystemRecoveryStageError::Inspect { stage, source })?;
    if !metadata.is_file() || FileIdentity::from(&metadata) != admitted.identity {
        return Err(FilesystemRecoveryStageError::Replaced { stage });
    }
    verify_length(stage, admitted.metadata.length(), metadata.len())
}

fn verify_current_entry(
    directory: &Dir,
    name: &str,
    stage: RecoveryStage,
    admitted: &AdmittedStage,
) -> Result<(), FilesystemRecoveryStageError> {
    let file = directory
        .open_with(name, &read_options())
        .map_err(|source| FilesystemRecoveryStageError::VerifyEntry { stage, source })?;
    let metadata = file
        .metadata()
        .map_err(|source| FilesystemRecoveryStageError::VerifyEntry { stage, source })?;
    if !metadata.is_file() || FileIdentity::from(&metadata) != admitted.identity {
        return Err(FilesystemRecoveryStageError::Replaced { stage });
    }
    verify_length(stage, admitted.metadata.length(), metadata.len())
}

const fn verify_length(
    stage: RecoveryStage,
    expected: RecoveryStageLength,
    observed: u64,
) -> Result<(), FilesystemRecoveryStageError> {
    if observed == expected.get() {
        Ok(())
    } else {
        Err(FilesystemRecoveryStageError::LengthChanged {
            stage,
            expected,
            observed,
        })
    }
}

fn read_options() -> OpenOptions {
    let mut options = OpenOptions::new();
    options.read(true).follow(FollowSymlinks::No).nonblock(true);
    options
}

fn read_write_options() -> OpenOptions {
    let mut options = OpenOptions::new();
    options
        .read(true)
        .write(true)
        .follow(FollowSymlinks::No)
        .nonblock(true);
    options
}
