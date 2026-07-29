//! This module owns exact filesystem recovery-candidate revalidation.

use std::io;

use super::{
    FilesystemRecoveryNextHeadFinalizer, FilesystemRecoveryStageError,
    RecoveryNextHeadFinalizationRequest, RecoveryNextHeadFinalizationStorageError,
    RecoveryNextHeadFinalizationTarget, RecoveryStage, RecoveryStageEvidence,
    RecoveryStageNamespacePhase, catalog_restart_loader, filesystem_recovery_stage,
};

const NEXT_HEAD: &str = "head.next";

pub(super) fn verify(
    finalizer: &FilesystemRecoveryNextHeadFinalizer,
    request: RecoveryNextHeadFinalizationRequest,
) -> Result<(), RecoveryNextHeadFinalizationStorageError> {
    verify_evidence(finalizer, request)?;
    let loaded =
        catalog_restart_loader::load_from_directory(finalizer.root(), NEXT_HEAD, finalizer.policy)
            .map_err(
                |source| RecoveryNextHeadFinalizationStorageError::CandidateView {
                    source: Box::new(source),
                },
            )?;
    let snapshot = loaded.snapshot().map_err(|source| {
        RecoveryNextHeadFinalizationStorageError::CandidateView {
            source: Box::new(source),
        }
    })?;
    let observed = RecoveryNextHeadFinalizationTarget::from_snapshot(&snapshot);
    if observed != request.target() {
        return Err(
            RecoveryNextHeadFinalizationStorageError::CandidateMismatch {
                expected: request.target(),
                observed,
            },
        );
    }
    verify_evidence(finalizer, request)
}

pub(super) fn synchronize(
    finalizer: &FilesystemRecoveryNextHeadFinalizer,
    request: RecoveryNextHeadFinalizationRequest,
) -> Result<(), RecoveryNextHeadFinalizationStorageError> {
    verify_namespaces(finalizer, RecoveryStageNamespacePhase::BeforeObservation)?;
    let stage = RecoveryStage::NextHead;
    let observed = filesystem_recovery_stage::observe_named(finalizer.root(), NEXT_HEAD, stage)
        .map_err(|source| map_stage(source, request.evidence()))?;
    require_evidence(request.evidence(), observed.evidence())?;
    observed
        .synchronize(finalizer.root(), NEXT_HEAD, stage)
        .map_err(|source| map_stage(source, request.evidence()))?;
    verify_namespaces(finalizer, RecoveryStageNamespacePhase::AfterObservation)?;
    verify(finalizer, request)
}

pub(super) fn verify_namespaces(
    finalizer: &FilesystemRecoveryNextHeadFinalizer,
    phase: RecoveryStageNamespacePhase,
) -> Result<(), RecoveryNextHeadFinalizationStorageError> {
    finalizer
        .discarder
        .inventory
        .verify_stage_namespaces(RecoveryStage::NextHead, phase)
        .map_err(stage)
}

fn verify_evidence(
    finalizer: &FilesystemRecoveryNextHeadFinalizer,
    request: RecoveryNextHeadFinalizationRequest,
) -> Result<(), RecoveryNextHeadFinalizationStorageError> {
    let observed = finalizer
        .discarder
        .inventory
        .fingerprint_stage(RecoveryStage::NextHead)
        .map_err(|source| map_stage(source, request.evidence()))?;
    let expected = request.evidence();
    require_evidence(expected, observed)
}

fn require_evidence(
    expected: RecoveryStageEvidence,
    observed: RecoveryStageEvidence,
) -> Result<(), RecoveryNextHeadFinalizationStorageError> {
    if observed == expected {
        Ok(())
    } else {
        Err(RecoveryNextHeadFinalizationStorageError::EvidenceMismatch { expected, observed })
    }
}

fn map_stage(
    source: FilesystemRecoveryStageError,
    expected: RecoveryStageEvidence,
) -> RecoveryNextHeadFinalizationStorageError {
    match source {
        FilesystemRecoveryStageError::Open { source, .. }
            if source.kind() == io::ErrorKind::NotFound =>
        {
            RecoveryNextHeadFinalizationStorageError::MissingCandidate { expected }
        }
        source => stage(source),
    }
}

fn stage(source: FilesystemRecoveryStageError) -> RecoveryNextHeadFinalizationStorageError {
    RecoveryNextHeadFinalizationStorageError::Stage {
        source: Box::new(source),
    }
}
