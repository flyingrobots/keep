//! This module owns exact writable recovery-stage materialization.

use std::io::{Read, Seek, SeekFrom};

use cap_std::fs::File;

use super::{FilesystemRecoveryStageError, RecoveryStage, RecoveryStageLength};

pub(super) fn read_and_position(
    file: &mut File,
    stage: RecoveryStage,
    length: RecoveryStageLength,
) -> Result<Box<[u8]>, FilesystemRecoveryStageError> {
    let mut encoded = allocate(stage, length)?;
    file.seek(SeekFrom::Start(0))
        .map_err(|source| FilesystemRecoveryStageError::Position { stage, source })?;
    file.read_exact(&mut encoded)
        .map_err(|source| FilesystemRecoveryStageError::Materialize {
            stage,
            expected: length,
            source,
        })?;
    verify_position(file, stage, length)?;
    Ok(encoded.into_boxed_slice())
}

fn allocate(
    stage: RecoveryStage,
    length: RecoveryStageLength,
) -> Result<Vec<u8>, FilesystemRecoveryStageError> {
    let host_length = usize::try_from(length.get()).map_err(|_source| {
        FilesystemRecoveryStageError::MaterializeAddressSpace {
            stage,
            byte_count: length.get(),
        }
    })?;
    let mut encoded = Vec::new();
    encoded.try_reserve_exact(host_length).map_err(|source| {
        FilesystemRecoveryStageError::MaterializeAllocation {
            stage,
            byte_count: length.get(),
            source,
        }
    })?;
    encoded.resize(host_length, 0);
    Ok(encoded)
}

pub(super) fn verify_position(
    file: &mut File,
    stage: RecoveryStage,
    expected: RecoveryStageLength,
) -> Result<(), FilesystemRecoveryStageError> {
    let observed = file
        .stream_position()
        .map_err(|source| FilesystemRecoveryStageError::Position { stage, source })?;
    if observed == expected.get() {
        Ok(())
    } else {
        Err(FilesystemRecoveryStageError::PositionMismatch {
            stage,
            expected,
            observed,
        })
    }
}
