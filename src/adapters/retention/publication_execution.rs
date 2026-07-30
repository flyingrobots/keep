//! This boundary module owns ordered retention publication execution.

use std::io;

use super::{
    PreparedRetentionPublication, RetentionNamespaceAdmission, RetentionPublicationError,
    RetentionPublicationPhase, RetentionPublicationPreparation, RetentionPublicationReceipt,
    RetentionPublicationStorage, RetentionTransitionDisposition,
};

/// Executes one prepared retention transition under revalidated authority.
///
/// Exact already-committed state performs no publication mutation. A new
/// publication returns only after head visibility and cleanup are synchronized.
///
/// # Errors
///
/// Returns [`RetentionPublicationError`] for current-state revalidation,
/// disposition disagreement, missing private artifacts, or the exact failed
/// durability phase. Failure returns no receipt.
pub fn execute_retention_publication(
    storage: &mut impl RetentionPublicationStorage,
    preparation: &RetentionPublicationPreparation<'_>,
) -> Result<RetentionPublicationReceipt, RetentionPublicationError> {
    let observed = storage
        .verify_current(preparation)
        .map_err(|source| RetentionPublicationError::CurrentVerification { source })?;
    if observed == RetentionTransitionDisposition::AlreadyCommitted {
        return Ok(RetentionPublicationReceipt::already_committed(preparation));
    }
    if preparation.disposition() != RetentionTransitionDisposition::Publish {
        return Err(RetentionPublicationError::DispositionMismatch {
            prepared: preparation.disposition(),
            observed,
        });
    }
    let publication = preparation
        .publication()
        .ok_or(RetentionPublicationError::MissingPublicationArtifacts)?;
    let namespace_admission = execute_root(storage, preparation)?;
    execute_manifest(storage, publication)?;
    execute_head(storage, publication)?;
    execute_cleanup(storage)?;
    Ok(RetentionPublicationReceipt::published(
        namespace_admission,
        preparation,
    ))
}

fn execute_root(
    storage: &mut impl RetentionPublicationStorage,
    preparation: &RetentionPublicationPreparation<'_>,
) -> Result<RetentionNamespaceAdmission, RetentionPublicationError> {
    let root = preparation.candidate();
    require(
        storage.write_root_stage(root),
        RetentionPublicationPhase::WriteRootStage,
    )?;
    require(
        storage.synchronize_root_stage(),
        RetentionPublicationPhase::SynchronizeRootStage,
    )?;
    let admission = require(
        storage.admit_root_namespace(root),
        RetentionPublicationPhase::AdmitRootNamespace,
    )?;
    if admission == RetentionNamespaceAdmission::Created {
        require(
            storage.synchronize_roots_after_namespace(),
            RetentionPublicationPhase::SynchronizeRootsAfterNamespace,
        )?;
    }
    require(storage.link_root(root), RetentionPublicationPhase::LinkRoot)?;
    require(
        storage.synchronize_root_namespace(root),
        RetentionPublicationPhase::SynchronizeRootNamespace,
    )?;
    Ok(admission)
}

fn execute_manifest(
    storage: &mut impl RetentionPublicationStorage,
    publication: &PreparedRetentionPublication,
) -> Result<(), RetentionPublicationError> {
    let manifest = publication.manifest();
    require(
        storage.write_manifest_stage(manifest),
        RetentionPublicationPhase::WriteManifestStage,
    )?;
    require(
        storage.synchronize_manifest_stage(),
        RetentionPublicationPhase::SynchronizeManifestStage,
    )?;
    require(
        storage.link_manifest(manifest),
        RetentionPublicationPhase::LinkManifest,
    )?;
    require(
        storage.synchronize_manifest_pool(),
        RetentionPublicationPhase::SynchronizeManifestPool,
    )
}

fn execute_head(
    storage: &mut impl RetentionPublicationStorage,
    publication: &PreparedRetentionPublication,
) -> Result<(), RetentionPublicationError> {
    require(
        storage.write_head_stage(publication.head()),
        RetentionPublicationPhase::WriteHeadStage,
    )?;
    require(
        storage.synchronize_head_stage(),
        RetentionPublicationPhase::SynchronizeHeadStage,
    )?;
    require(
        storage.replace_head(),
        RetentionPublicationPhase::ReplaceHead,
    )?;
    require(
        storage.synchronize_retention_namespace(),
        RetentionPublicationPhase::SynchronizeRetentionNamespace,
    )
}

fn execute_cleanup(
    storage: &mut impl RetentionPublicationStorage,
) -> Result<(), RetentionPublicationError> {
    require(
        storage.remove_root_stage(),
        RetentionPublicationPhase::RemoveRootStage,
    )?;
    require(
        storage.remove_manifest_stage(),
        RetentionPublicationPhase::RemoveManifestStage,
    )?;
    require(
        storage.synchronize_cleanup(),
        RetentionPublicationPhase::SynchronizeCleanup,
    )
}

fn require<T>(
    result: io::Result<T>,
    phase: RetentionPublicationPhase,
) -> Result<T, RetentionPublicationError> {
    result.map_err(|source| RetentionPublicationError::Storage { phase, source })
}
