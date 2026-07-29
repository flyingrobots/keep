//! Ordered, retry-safe stage-discard execution laws.

use std::error::Error;
use std::io;

use keep::{
    RecoveryStage, RecoveryStageDiscardError, RecoveryStageDiscardOutcome,
    RecoveryStageDiscardStorageError, RecoveryStageParent, execute_recovery_stage_discard,
};

use super::storage_double::{Operation, StageDiscardDouble};
use super::{discard_request, evidence, truncated_fixture};

#[test]
fn exact_evidence_is_removed_before_its_parent_is_synchronized() -> Result<(), Box<dyn Error>> {
    let bytes = truncated_fixture(RecoveryStage::Segment)?;
    let request = discard_request(RecoveryStage::Segment, &bytes)?;
    let mut storage = StageDiscardDouble::new(Some(request.evidence()));

    let receipt = execute_recovery_stage_discard(&mut storage, request)?;

    assert_eq!(receipt.evidence(), request.evidence());
    assert_eq!(receipt.reason(), request.reason());
    assert_eq!(receipt.outcome(), RecoveryStageDiscardOutcome::Removed);
    assert_eq!(
        storage.operations(),
        &[
            Operation::Remove(request.evidence()),
            Operation::Synchronize(RecoveryStageParent::Staging),
        ]
    );
    assert_eq!(storage.present(), None);
    Ok(())
}

#[test]
fn absent_exact_retry_still_synchronizes_the_selected_parent() -> Result<(), Box<dyn Error>> {
    for stage in [
        RecoveryStage::Segment,
        RecoveryStage::Catalog,
        RecoveryStage::NextHead,
    ] {
        let bytes = truncated_fixture(stage)?;
        let request = discard_request(stage, &bytes)?;
        let mut storage = StageDiscardDouble::new(None);

        let receipt = execute_recovery_stage_discard(&mut storage, request)?;

        assert_eq!(
            receipt.outcome(),
            RecoveryStageDiscardOutcome::AlreadyAbsent
        );
        assert_eq!(
            storage.operations(),
            &[
                Operation::Remove(request.evidence()),
                Operation::Synchronize(expected_parent(stage)),
            ]
        );
    }
    Ok(())
}

#[test]
fn changed_evidence_refuses_without_removal_or_parent_sync() -> Result<(), Box<dyn Error>> {
    let bytes = truncated_fixture(RecoveryStage::Segment)?;
    let changed = [1_u8];
    let request = discard_request(RecoveryStage::Segment, &bytes)?;
    let observed = evidence(RecoveryStage::Segment, &changed)?;
    let mut storage = StageDiscardDouble::new(Some(observed));

    let error = execute_recovery_stage_discard(&mut storage, request)
        .err()
        .ok_or("changed stage evidence was discarded")?;

    assert!(matches!(
        error,
        RecoveryStageDiscardError::Remove {
            source: RecoveryStageDiscardStorageError::EvidenceMismatch {
                expected,
                observed: actual,
            },
        } if expected == request.evidence() && actual == observed
    ));
    assert_eq!(storage.present(), Some(observed));
    assert_eq!(
        storage.operations(),
        &[Operation::Remove(request.evidence())]
    );
    Ok(())
}

#[test]
fn retry_after_remove_before_parent_sync_is_idempotent() -> Result<(), Box<dyn Error>> {
    let bytes = truncated_fixture(RecoveryStage::NextHead)?;
    let request = discard_request(RecoveryStage::NextHead, &bytes)?;
    let mut storage = StageDiscardDouble::new(Some(request.evidence())).fail_next_synchronization();

    let error = execute_recovery_stage_discard(&mut storage, request)
        .err()
        .ok_or("injected parent synchronization failure was ignored")?;

    assert!(matches!(
        error,
        RecoveryStageDiscardError::Synchronize {
            stage: RecoveryStage::NextHead,
            source,
        } if source.kind() == io::ErrorKind::Other
    ));
    assert_eq!(storage.present(), None);

    let receipt = execute_recovery_stage_discard(&mut storage, request)?;

    assert_eq!(
        receipt.outcome(),
        RecoveryStageDiscardOutcome::AlreadyAbsent
    );
    assert_eq!(
        storage.operations(),
        &[
            Operation::Remove(request.evidence()),
            Operation::Synchronize(RecoveryStageParent::Root),
            Operation::Remove(request.evidence()),
            Operation::Synchronize(RecoveryStageParent::Root),
        ]
    );
    Ok(())
}

const fn expected_parent(stage: RecoveryStage) -> RecoveryStageParent {
    match stage {
        RecoveryStage::Segment | RecoveryStage::Catalog => RecoveryStageParent::Staging,
        RecoveryStage::NextHead => RecoveryStageParent::Root,
    }
}
