//! This module owns exact recovery-stage file synchronization.

use std::io;

use cap_std::fs::Dir;

use super::{
    FilesystemRecoveryStageError, RecoveryStageCompletionStorageError, RecoveryStageEvidence,
    RecoveryStageSynchronizationOutcome, filesystem_recovery_stage,
};

pub(super) fn synchronize_if_matching(
    directory: &Dir,
    expected: RecoveryStageEvidence,
) -> Result<RecoveryStageSynchronizationOutcome, RecoveryStageCompletionStorageError> {
    synchronize_with(directory, expected, || {})
}

#[cfg(test)]
pub(super) fn synchronize_if_matching_with<F>(
    directory: &Dir,
    expected: RecoveryStageEvidence,
    after_open: F,
) -> Result<RecoveryStageSynchronizationOutcome, RecoveryStageCompletionStorageError>
where
    F: FnOnce(),
{
    synchronize_with(directory, expected, after_open)
}

fn synchronize_with<F>(
    directory: &Dir,
    expected: RecoveryStageEvidence,
    after_open: F,
) -> Result<RecoveryStageSynchronizationOutcome, RecoveryStageCompletionStorageError>
where
    F: FnOnce(),
{
    let stage = expected.stage();
    match directory.symlink_metadata(stage.file_name()) {
        Ok(_) => {}
        Err(source) if source.kind() == io::ErrorKind::NotFound => {
            return Ok(RecoveryStageSynchronizationOutcome::AlreadyAbsent);
        }
        Err(source) => {
            return Err(RecoveryStageCompletionStorageError::storage(source));
        }
    }
    let observed = filesystem_recovery_stage::observe_named_with(
        directory,
        stage.file_name(),
        stage,
        after_open,
    )
    .map_err(stage_error)?;
    require_evidence(expected, observed.evidence())?;
    observed
        .synchronize(directory, stage.file_name(), stage)
        .map_err(stage_error)?;
    let verified = filesystem_recovery_stage::fingerprint(directory, stage).map_err(stage_error)?;
    require_evidence(expected, verified)?;
    Ok(RecoveryStageSynchronizationOutcome::Synchronized)
}

fn require_evidence(
    expected: RecoveryStageEvidence,
    observed: RecoveryStageEvidence,
) -> Result<(), RecoveryStageCompletionStorageError> {
    if observed == expected {
        Ok(())
    } else {
        Err(RecoveryStageCompletionStorageError::EvidenceMismatch { expected, observed })
    }
}

fn stage_error(source: FilesystemRecoveryStageError) -> RecoveryStageCompletionStorageError {
    RecoveryStageCompletionStorageError::storage(io::Error::other(source))
}
