//! This module owns filesystem immutable-pool recovery transitions.

use std::io;

use cap_std::fs::Dir;

use super::{
    FilesystemRecoveryInventoryReader, FilesystemRecoveryStageCompleter,
    FilesystemRecoveryStageError, RecoveryStageCompletionPool, RecoveryStageCompletionRequest,
    RecoveryStageCompletionStorageError, RecoveryStageCompletionTarget, RecoveryStageEvidence,
    RecoveryStageNamespacePhase, RecoveryStagePoolOutcome, filesystem_catalog_artifact,
    filesystem_recovery_stage, physical_pool_name,
};

pub(super) fn link_or_admit(
    inventory: &FilesystemRecoveryInventoryReader,
    request: RecoveryStageCompletionRequest,
) -> Result<RecoveryStagePoolOutcome, RecoveryStageCompletionStorageError> {
    verify_namespaces(
        inventory,
        request,
        RecoveryStageNamespacePhase::BeforeObservation,
    )?;
    let stage = request.evidence().stage();
    let stage_directory = inventory.stage_directory(stage);
    let pool_directory = inventory.completion_pool_directory(request.pool());
    let pool_name = pool_name(request.target());
    let outcome = if entry_is_absent(stage_directory, stage.file_name())? {
        if entry_is_absent(pool_directory, &pool_name)? {
            return Err(RecoveryStageCompletionStorageError::Missing { request });
        }
        RecoveryStagePoolOutcome::AlreadyPresent
    } else {
        let observed =
            filesystem_recovery_stage::fingerprint(stage_directory, stage).map_err(stage_error)?;
        require_evidence(request.evidence(), observed)?;
        link(
            stage_directory,
            stage.file_name(),
            pool_directory,
            &pool_name,
        )?
    };
    verify_namespaces(
        inventory,
        request,
        RecoveryStageNamespacePhase::AfterObservation,
    )?;
    Ok(outcome)
}

pub(super) fn verify(
    inventory: &FilesystemRecoveryInventoryReader,
    request: RecoveryStageCompletionRequest,
) -> Result<(), RecoveryStageCompletionStorageError> {
    verify_namespaces(
        inventory,
        request,
        RecoveryStageNamespacePhase::BeforeObservation,
    )?;
    let observed = filesystem_recovery_stage::fingerprint_named(
        inventory.completion_pool_directory(request.pool()),
        &pool_name(request.target()),
        request.evidence().stage(),
    )
    .map_err(stage_error)?;
    require_evidence(request.evidence(), observed)?;
    verify_namespaces(
        inventory,
        request,
        RecoveryStageNamespacePhase::AfterObservation,
    )
}

pub(super) fn synchronize(
    inventory: &FilesystemRecoveryInventoryReader,
    pool: RecoveryStageCompletionPool,
) -> io::Result<()> {
    filesystem_catalog_artifact::synchronize_directory(inventory.completion_pool_directory(pool))
}

impl FilesystemRecoveryStageCompleter {
    #[cfg(test)]
    pub(super) fn verify_pool_with<F>(
        &self,
        request: RecoveryStageCompletionRequest,
        after_open: F,
    ) -> Result<(), RecoveryStageCompletionStorageError>
    where
        F: FnOnce(),
    {
        verify_with(&self.discarder.inventory, request, after_open)
    }
}

#[cfg(test)]
fn verify_with<F>(
    inventory: &FilesystemRecoveryInventoryReader,
    request: RecoveryStageCompletionRequest,
    after_open: F,
) -> Result<(), RecoveryStageCompletionStorageError>
where
    F: FnOnce(),
{
    verify_namespaces(
        inventory,
        request,
        RecoveryStageNamespacePhase::BeforeObservation,
    )?;
    let observed = filesystem_recovery_stage::observe_named_with(
        inventory.completion_pool_directory(request.pool()),
        &pool_name(request.target()),
        request.evidence().stage(),
        after_open,
    )
    .map_err(stage_error)?
    .evidence();
    require_evidence(request.evidence(), observed)?;
    verify_namespaces(
        inventory,
        request,
        RecoveryStageNamespacePhase::AfterObservation,
    )
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

fn link(
    stage_directory: &Dir,
    stage_name: &str,
    pool_directory: &Dir,
    pool_name: &str,
) -> Result<RecoveryStagePoolOutcome, RecoveryStageCompletionStorageError> {
    match stage_directory.hard_link(stage_name, pool_directory, pool_name) {
        Ok(()) => Ok(RecoveryStagePoolOutcome::Linked),
        Err(source) if source.kind() == io::ErrorKind::AlreadyExists => {
            Ok(RecoveryStagePoolOutcome::AlreadyPresent)
        }
        Err(source) => Err(RecoveryStageCompletionStorageError::storage(source)),
    }
}

fn entry_is_absent(
    directory: &Dir,
    name: &str,
) -> Result<bool, RecoveryStageCompletionStorageError> {
    match directory.symlink_metadata(name) {
        Ok(_) => Ok(false),
        Err(source) if source.kind() == io::ErrorKind::NotFound => Ok(true),
        Err(source) => Err(RecoveryStageCompletionStorageError::storage(source)),
    }
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

fn pool_name(target: RecoveryStageCompletionTarget) -> String {
    match target {
        RecoveryStageCompletionTarget::Segment { digest } => physical_pool_name::segment(digest),
        RecoveryStageCompletionTarget::Catalog {
            generation, digest, ..
        } => physical_pool_name::catalog(generation, digest),
    }
}

fn stage_error(source: FilesystemRecoveryStageError) -> RecoveryStageCompletionStorageError {
    RecoveryStageCompletionStorageError::storage(io::Error::other(source))
}
