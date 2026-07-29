//! This module owns filesystem execution of exact stage completion.

use std::io;

use super::{
    FilesystemRecoveryInventoryReader, FilesystemRecoveryStageCompleter,
    FilesystemRecoveryStageError, RecoveryStageCompletionPool, RecoveryStageCompletionRequest,
    RecoveryStageCompletionStorage, RecoveryStageCompletionStorageError,
    RecoveryStageDiscardOutcome, RecoveryStageDiscardStorageError, RecoveryStageEvidence,
    RecoveryStageNamespacePhase, RecoveryStageParent, RecoveryStagePoolOutcome,
    RecoveryStageSynchronizationOutcome, filesystem_catalog_artifact,
    filesystem_recovery_stage_completion_pool, filesystem_recovery_stage_discard_storage,
    filesystem_recovery_stage_sync,
};

impl RecoveryStageCompletionStorage for FilesystemRecoveryStageCompleter {
    fn synchronize_stage_if_present(
        &mut self,
        request: RecoveryStageCompletionRequest,
    ) -> Result<RecoveryStageSynchronizationOutcome, RecoveryStageCompletionStorageError> {
        let inventory = &self.discarder.inventory;
        verify_before(inventory, request)?;
        let outcome = filesystem_recovery_stage_sync::synchronize_if_matching(
            inventory.stage_directory(request.evidence().stage()),
            request.evidence(),
        )?;
        verify_after(inventory, request)?;
        Ok(outcome)
    }

    fn link_stage_or_admit_pool(
        &mut self,
        request: RecoveryStageCompletionRequest,
    ) -> Result<RecoveryStagePoolOutcome, RecoveryStageCompletionStorageError> {
        filesystem_recovery_stage_completion_pool::link_or_admit(&self.discarder.inventory, request)
    }

    fn verify_pool(
        &mut self,
        request: RecoveryStageCompletionRequest,
    ) -> Result<(), RecoveryStageCompletionStorageError> {
        filesystem_recovery_stage_completion_pool::verify(&self.discarder.inventory, request)
    }

    fn synchronize_pool(&mut self, pool: RecoveryStageCompletionPool) -> io::Result<()> {
        filesystem_recovery_stage_completion_pool::synchronize(&self.discarder.inventory, pool)
    }

    fn remove_stage_if_matching(
        &mut self,
        expected: RecoveryStageEvidence,
    ) -> Result<RecoveryStageDiscardOutcome, RecoveryStageDiscardStorageError> {
        filesystem_recovery_stage_discard_storage::remove_if_matching(
            &self.discarder.inventory,
            expected,
        )
    }

    fn synchronize_staging(&mut self) -> io::Result<()> {
        filesystem_catalog_artifact::synchronize_directory(
            self.discarder
                .inventory
                .parent_directory(RecoveryStageParent::Staging),
        )
    }
}

impl FilesystemRecoveryStageCompleter {
    #[cfg(test)]
    pub(super) fn synchronize_stage_if_present_with<F>(
        &self,
        request: RecoveryStageCompletionRequest,
        after_open: F,
    ) -> Result<RecoveryStageSynchronizationOutcome, RecoveryStageCompletionStorageError>
    where
        F: FnOnce(),
    {
        synchronize_stage_with(&self.discarder.inventory, request, after_open)
    }
}

#[cfg(test)]
fn synchronize_stage_with<F>(
    inventory: &FilesystemRecoveryInventoryReader,
    request: RecoveryStageCompletionRequest,
    after_open: F,
) -> Result<RecoveryStageSynchronizationOutcome, RecoveryStageCompletionStorageError>
where
    F: FnOnce(),
{
    verify_before(inventory, request)?;
    let outcome = filesystem_recovery_stage_sync::synchronize_if_matching_with(
        inventory.stage_directory(request.evidence().stage()),
        request.evidence(),
        after_open,
    )?;
    verify_after(inventory, request)?;
    Ok(outcome)
}

fn verify_before(
    inventory: &FilesystemRecoveryInventoryReader,
    request: RecoveryStageCompletionRequest,
) -> Result<(), RecoveryStageCompletionStorageError> {
    verify_namespaces(
        inventory,
        request,
        RecoveryStageNamespacePhase::BeforeObservation,
    )
}

fn verify_after(
    inventory: &FilesystemRecoveryInventoryReader,
    request: RecoveryStageCompletionRequest,
) -> Result<(), RecoveryStageCompletionStorageError> {
    verify_namespaces(
        inventory,
        request,
        RecoveryStageNamespacePhase::AfterObservation,
    )
}

fn verify_namespaces(
    inventory: &FilesystemRecoveryInventoryReader,
    request: RecoveryStageCompletionRequest,
    phase: RecoveryStageNamespacePhase,
) -> Result<(), RecoveryStageCompletionStorageError> {
    inventory
        .verify_stage_namespaces(request.evidence().stage(), phase)
        .map_err(stage_error)
}

fn stage_error(source: FilesystemRecoveryStageError) -> RecoveryStageCompletionStorageError {
    RecoveryStageCompletionStorageError::storage(io::Error::other(source))
}
