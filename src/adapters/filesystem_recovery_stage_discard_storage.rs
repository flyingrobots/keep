//! This module owns filesystem execution of exact recovery-stage discard.

use std::io;

use cap_std::fs::Dir;

use super::{
    FilesystemRecoveryInventoryReader, FilesystemRecoveryStageDiscarder,
    FilesystemRecoveryStageError, RecoveryStage, RecoveryStageDiscardOutcome,
    RecoveryStageDiscardStorage, RecoveryStageDiscardStorageError, RecoveryStageEvidence,
    RecoveryStageNamespacePhase, RecoveryStageParent, filesystem_catalog_artifact,
    filesystem_recovery_stage,
};

impl RecoveryStageDiscardStorage for FilesystemRecoveryStageDiscarder {
    fn remove_if_matching(
        &mut self,
        expected: RecoveryStageEvidence,
    ) -> Result<RecoveryStageDiscardOutcome, RecoveryStageDiscardStorageError> {
        remove_with(
            &self.inventory,
            expected,
            filesystem_recovery_stage::fingerprint,
        )
    }

    fn synchronize_parent(&mut self, parent: RecoveryStageParent) -> io::Result<()> {
        filesystem_catalog_artifact::synchronize_directory(self.inventory.parent_directory(parent))
    }
}

impl FilesystemRecoveryStageDiscarder {
    #[cfg(test)]
    pub(super) fn remove_if_matching_with<F>(
        &self,
        expected: RecoveryStageEvidence,
        after_open: F,
    ) -> Result<RecoveryStageDiscardOutcome, RecoveryStageDiscardStorageError>
    where
        F: FnOnce(),
    {
        remove_with(&self.inventory, expected, |directory, stage| {
            filesystem_recovery_stage::fingerprint_with(directory, stage, after_open)
        })
    }
}

pub(super) fn remove_if_matching(
    inventory: &FilesystemRecoveryInventoryReader,
    expected: RecoveryStageEvidence,
) -> Result<RecoveryStageDiscardOutcome, RecoveryStageDiscardStorageError> {
    remove_with(inventory, expected, filesystem_recovery_stage::fingerprint)
}

fn remove_with<F>(
    inventory: &FilesystemRecoveryInventoryReader,
    expected: RecoveryStageEvidence,
    observe: F,
) -> Result<RecoveryStageDiscardOutcome, RecoveryStageDiscardStorageError>
where
    F: FnOnce(&Dir, RecoveryStage) -> Result<RecoveryStageEvidence, FilesystemRecoveryStageError>,
{
    let stage = expected.stage();
    inventory
        .verify_stage_namespaces(stage, RecoveryStageNamespacePhase::BeforeObservation)
        .map_err(stage_error)?;
    let directory = inventory.stage_directory(stage);
    if stage_is_absent(directory, stage)? {
        inventory
            .verify_stage_namespaces(stage, RecoveryStageNamespacePhase::AfterObservation)
            .map_err(stage_error)?;
        return Ok(RecoveryStageDiscardOutcome::AlreadyAbsent);
    }
    let observed = observe(directory, stage).map_err(stage_error)?;
    if observed != expected {
        return Err(RecoveryStageDiscardStorageError::EvidenceMismatch { expected, observed });
    }
    inventory
        .verify_stage_namespaces(stage, RecoveryStageNamespacePhase::AfterObservation)
        .map_err(stage_error)?;
    directory
        .remove_file(stage.file_name())
        .map_err(storage_error)?;
    Ok(RecoveryStageDiscardOutcome::Removed)
}

fn stage_is_absent(
    directory: &Dir,
    stage: RecoveryStage,
) -> Result<bool, RecoveryStageDiscardStorageError> {
    match directory.symlink_metadata(stage.file_name()) {
        Ok(_) => Ok(false),
        Err(source) if source.kind() == io::ErrorKind::NotFound => Ok(true),
        Err(source) => Err(storage_error(source)),
    }
}

fn stage_error(source: FilesystemRecoveryStageError) -> RecoveryStageDiscardStorageError {
    storage_error(io::Error::other(source))
}

const fn storage_error(source: io::Error) -> RecoveryStageDiscardStorageError {
    RecoveryStageDiscardStorageError::Storage { source }
}
