//! Process-death retry laws for complete-stage recovery.

use std::error::Error;

use keep::{
    RecoveryStage, RecoveryStageCompletionError, RecoveryStageDiscardOutcome,
    RecoveryStagePoolOutcome, execute_recovery_stage_completion,
};

use super::storage_double::StageCompletionDouble;
use super::{SEGMENT_HEX, completion_request, fixture};

#[test]
fn stage_sync_failure_stops_before_link_or_pool_admission() -> Result<(), Box<dyn Error>> {
    let bytes = fixture(SEGMENT_HEX)?;
    let request = completion_request(RecoveryStage::Segment, &bytes)?;
    let mut storage = StageCompletionDouble::new(Some(request.evidence()), None)
        .fail_next_stage_synchronization();

    let error = execute_recovery_stage_completion(&mut storage, request)
        .err()
        .ok_or("injected stage synchronization failure was ignored")?;

    assert!(matches!(
        error,
        RecoveryStageCompletionError::SynchronizeStage {
            stage: RecoveryStage::Segment,
            ..
        }
    ));
    assert_eq!(storage.pool(), None);
    assert_eq!(storage.stage(), Some(request.evidence()));
    assert_eq!(
        storage.operations(),
        &[super::storage_double::Operation::SynchronizeStage(request)]
    );
    Ok(())
}

#[test]
fn retry_after_pool_sync_failure_reverifies_before_stage_removal() -> Result<(), Box<dyn Error>> {
    let bytes = fixture(SEGMENT_HEX)?;
    let request = completion_request(RecoveryStage::Segment, &bytes)?;
    let mut storage =
        StageCompletionDouble::new(Some(request.evidence()), None).fail_next_pool_synchronization();

    let error = execute_recovery_stage_completion(&mut storage, request)
        .err()
        .ok_or("injected pool synchronization failure was ignored")?;

    assert!(matches!(
        error,
        RecoveryStageCompletionError::SynchronizePool { pool, .. }
            if pool == request.pool()
    ));
    assert_eq!(storage.pool(), Some(request));
    assert_eq!(storage.stage(), Some(request.evidence()));

    let receipt = execute_recovery_stage_completion(&mut storage, request)?;

    assert_eq!(
        receipt.pool_outcome(),
        RecoveryStagePoolOutcome::AlreadyPresent
    );
    assert_eq!(
        receipt.stage_outcome(),
        RecoveryStageDiscardOutcome::Removed
    );
    Ok(())
}

#[test]
fn retry_after_stage_removal_reestablishes_staging_durability() -> Result<(), Box<dyn Error>> {
    let bytes = fixture(SEGMENT_HEX)?;
    let request = completion_request(RecoveryStage::Segment, &bytes)?;
    let mut storage = StageCompletionDouble::new(Some(request.evidence()), None)
        .fail_next_staging_synchronization();

    let error = execute_recovery_stage_completion(&mut storage, request)
        .err()
        .ok_or("injected staging synchronization failure was ignored")?;

    assert!(matches!(
        error,
        RecoveryStageCompletionError::SynchronizeStaging {
            stage: RecoveryStage::Segment,
            ..
        }
    ));
    assert_eq!(storage.pool(), Some(request));
    assert_eq!(storage.stage(), None);

    let receipt = execute_recovery_stage_completion(&mut storage, request)?;

    assert_eq!(
        receipt.pool_outcome(),
        RecoveryStagePoolOutcome::AlreadyPresent
    );
    assert_eq!(
        receipt.stage_outcome(),
        RecoveryStageDiscardOutcome::AlreadyAbsent
    );
    Ok(())
}
