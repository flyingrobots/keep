//! This module owns pinned filesystem recovery-stage observation.

use cap_fs_ext::{FollowSymlinks, MetadataExt, OpenOptionsFollowExt, OpenOptionsSyncExt};
use cap_std::fs::{Dir, File, Metadata, OpenOptions};

use super::{
    FilesystemRecoveryStageError, RecoveryStage, RecoveryStageEvidence, RecoveryStageLength,
    RecoveryStageMetadata, fingerprint_recovery_stage,
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

pub(super) fn fingerprint(
    directory: &Dir,
    stage: RecoveryStage,
) -> Result<RecoveryStageEvidence, FilesystemRecoveryStageError> {
    observe(directory, stage, || {})
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
    observe(directory, stage, after_open)
}

fn observe<F>(
    directory: &Dir,
    stage: RecoveryStage,
    after_open: F,
) -> Result<RecoveryStageEvidence, FilesystemRecoveryStageError>
where
    F: FnOnce(),
{
    let mut file = open_stage(directory, stage)?;
    let admitted = admit_stage(&file, stage)?;
    after_open();
    let evidence = fingerprint_recovery_stage(admitted.metadata, &mut file)
        .map_err(|source| FilesystemRecoveryStageError::Fingerprint { stage, source })?;
    verify_length(stage, admitted.metadata.length(), evidence.length().get())?;
    verify_opened_handle(&file, stage, &admitted)?;
    verify_current_entry(directory, stage, &admitted)?;
    Ok(evidence)
}

fn open_stage(directory: &Dir, stage: RecoveryStage) -> Result<File, FilesystemRecoveryStageError> {
    directory
        .open_with(stage.file_name(), &read_options())
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
    stage: RecoveryStage,
    admitted: &AdmittedStage,
) -> Result<(), FilesystemRecoveryStageError> {
    let file = directory
        .open_with(stage.file_name(), &read_options())
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
