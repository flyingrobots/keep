//! Ordered complete-stage recovery execution laws.

use std::error::Error;

use keep::{
    RecoveryStage, RecoveryStageCompletionPool, RecoveryStageDiscardOutcome,
    RecoveryStagePoolOutcome, RecoveryStageSynchronizationOutcome,
    execute_recovery_stage_completion,
};

use super::storage_double::{Operation, StageCompletionDouble};
use super::{CATALOG_HEX, SEGMENT_HEX, completion_request, fixture};

#[test]
fn complete_segment_becomes_a_durable_orphan_before_stage_removal() -> Result<(), Box<dyn Error>> {
    let bytes = fixture(SEGMENT_HEX)?;
    let request = completion_request(RecoveryStage::Segment, &bytes)?;
    let mut storage = StageCompletionDouble::new(Some(request.evidence()), None);

    let receipt = execute_recovery_stage_completion(&mut storage, request)?;

    assert_eq!(receipt.evidence(), request.evidence());
    assert_eq!(receipt.target(), request.target());
    assert_eq!(
        receipt.synchronization_outcome(),
        RecoveryStageSynchronizationOutcome::Synchronized
    );
    assert_eq!(receipt.pool_outcome(), RecoveryStagePoolOutcome::Linked);
    assert_eq!(
        receipt.stage_outcome(),
        RecoveryStageDiscardOutcome::Removed
    );
    assert_eq!(
        storage.operations(),
        &[
            Operation::SynchronizeStage(request),
            Operation::LinkOrAdmit(request),
            Operation::VerifyPool(request),
            Operation::SynchronizePool(RecoveryStageCompletionPool::Segments),
            Operation::RemoveStage(request.evidence()),
            Operation::SynchronizeStaging,
        ]
    );
    assert_eq!(storage.pool(), Some(request));
    assert_eq!(storage.stage(), None);
    Ok(())
}

#[test]
fn complete_catalog_selects_the_catalog_pool() -> Result<(), Box<dyn Error>> {
    let bytes = fixture(CATALOG_HEX)?;
    let request = completion_request(RecoveryStage::Catalog, &bytes)?;
    let mut storage = StageCompletionDouble::new(Some(request.evidence()), None);

    let receipt = execute_recovery_stage_completion(&mut storage, request)?;

    assert_eq!(receipt.pool_outcome(), RecoveryStagePoolOutcome::Linked);
    assert_eq!(
        storage.operations(),
        &[
            Operation::SynchronizeStage(request),
            Operation::LinkOrAdmit(request),
            Operation::VerifyPool(request),
            Operation::SynchronizePool(RecoveryStageCompletionPool::Catalogs),
            Operation::RemoveStage(request.evidence()),
            Operation::SynchronizeStaging,
        ]
    );
    Ok(())
}

#[test]
fn existing_exact_pool_is_verified_before_stage_removal() -> Result<(), Box<dyn Error>> {
    let bytes = fixture(SEGMENT_HEX)?;
    let request = completion_request(RecoveryStage::Segment, &bytes)?;
    let mut storage = StageCompletionDouble::new(Some(request.evidence()), Some(request));

    let receipt = execute_recovery_stage_completion(&mut storage, request)?;

    assert_eq!(
        receipt.pool_outcome(),
        RecoveryStagePoolOutcome::AlreadyPresent
    );
    assert_eq!(
        receipt.synchronization_outcome(),
        RecoveryStageSynchronizationOutcome::Synchronized
    );
    assert_eq!(
        receipt.stage_outcome(),
        RecoveryStageDiscardOutcome::Removed
    );
    assert_eq!(
        storage.operations(),
        &[
            Operation::SynchronizeStage(request),
            Operation::LinkOrAdmit(request),
            Operation::VerifyPool(request),
            Operation::SynchronizePool(RecoveryStageCompletionPool::Segments),
            Operation::RemoveStage(request.evidence()),
            Operation::SynchronizeStaging,
        ]
    );
    Ok(())
}

#[test]
fn absent_stage_with_exact_pool_is_an_idempotent_completed_retry() -> Result<(), Box<dyn Error>> {
    let bytes = fixture(CATALOG_HEX)?;
    let request = completion_request(RecoveryStage::Catalog, &bytes)?;
    let mut storage = StageCompletionDouble::new(None, Some(request));

    let receipt = execute_recovery_stage_completion(&mut storage, request)?;

    assert_eq!(
        receipt.pool_outcome(),
        RecoveryStagePoolOutcome::AlreadyPresent
    );
    assert_eq!(
        receipt.synchronization_outcome(),
        RecoveryStageSynchronizationOutcome::AlreadyAbsent
    );
    assert_eq!(
        receipt.stage_outcome(),
        RecoveryStageDiscardOutcome::AlreadyAbsent
    );
    assert_eq!(storage.pool(), Some(request));
    assert_eq!(storage.stage(), None);
    Ok(())
}

#[test]
fn conflicting_existing_pool_refuses_before_sync_or_stage_removal() -> Result<(), Box<dyn Error>> {
    let segment = fixture(SEGMENT_HEX)?;
    let catalog = fixture(CATALOG_HEX)?;
    let request = completion_request(RecoveryStage::Segment, &segment)?;
    let conflict = completion_request(RecoveryStage::Catalog, &catalog)?;
    let mut storage = StageCompletionDouble::new(Some(request.evidence()), Some(conflict));

    let error = execute_recovery_stage_completion(&mut storage, request)
        .err()
        .ok_or("conflicting pool artifact was accepted")?;

    assert!(matches!(
        error,
        keep::RecoveryStageCompletionError::VerifyPool { target, .. }
            if target == request.target()
    ));
    assert_eq!(
        storage.operations(),
        &[
            Operation::SynchronizeStage(request),
            Operation::LinkOrAdmit(request),
            Operation::VerifyPool(request),
        ]
    );
    assert_eq!(storage.stage(), Some(request.evidence()));
    assert_eq!(storage.pool(), Some(conflict));
    Ok(())
}
