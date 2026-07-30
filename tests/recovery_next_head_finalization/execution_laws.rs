//! Ordered and retry-safe next-head finalization laws.

use std::error::Error;

use keep::{
    RecoveryNextHeadFinalizationError, RecoveryNextHeadFinalizationOutcome,
    RecoveryNextHeadFinalizationReadiness, execute_recovery_next_head_finalization,
};

use super::storage_double::{NextHeadDouble, Operation};
use super::{CATALOG_ONE_HEX, HEAD_ONE_HEX, SEGMENT_HEX, fixture, initial_request};

#[test]
fn ready_candidate_replaces_head_before_root_synchronization() -> Result<(), Box<dyn Error>> {
    let segment = fixture(SEGMENT_HEX)?;
    let catalog = fixture(CATALOG_ONE_HEX)?;
    let head = fixture(HEAD_ONE_HEX)?;
    let request = initial_request(&head, &catalog, &segment)?;
    let mut storage = NextHeadDouble::new(RecoveryNextHeadFinalizationReadiness::Ready);

    let receipt = execute_recovery_next_head_finalization(&mut storage, request)?;

    assert_eq!(receipt.evidence(), request.evidence());
    assert_eq!(receipt.target(), request.target());
    assert_eq!(
        receipt.outcome(),
        RecoveryNextHeadFinalizationOutcome::Finalized
    );
    assert_eq!(
        storage.operations(),
        &[
            Operation::Verify(request),
            Operation::SynchronizeCandidate(request),
            Operation::Replace(request),
            Operation::SynchronizeRoot,
        ]
    );
    Ok(())
}

#[test]
fn already_current_retry_only_resynchronizes_root() -> Result<(), Box<dyn Error>> {
    let segment = fixture(SEGMENT_HEX)?;
    let catalog = fixture(CATALOG_ONE_HEX)?;
    let head = fixture(HEAD_ONE_HEX)?;
    let request = initial_request(&head, &catalog, &segment)?;
    let mut storage = NextHeadDouble::new(RecoveryNextHeadFinalizationReadiness::AlreadyFinalized);

    let receipt = execute_recovery_next_head_finalization(&mut storage, request)?;

    assert_eq!(
        receipt.outcome(),
        RecoveryNextHeadFinalizationOutcome::AlreadyFinalized
    );
    assert_eq!(
        storage.operations(),
        &[Operation::Verify(request), Operation::SynchronizeRoot]
    );
    Ok(())
}

#[test]
fn verification_and_replacement_failures_stop_later_mutation() -> Result<(), Box<dyn Error>> {
    let segment = fixture(SEGMENT_HEX)?;
    let catalog = fixture(CATALOG_ONE_HEX)?;
    let head = fixture(HEAD_ONE_HEX)?;
    let request = initial_request(&head, &catalog, &segment)?;
    let mut verify_failure =
        NextHeadDouble::new(RecoveryNextHeadFinalizationReadiness::Ready).fail_next_verification();
    let mut replace_failure =
        NextHeadDouble::new(RecoveryNextHeadFinalizationReadiness::Ready).fail_next_replacement();

    let verify_error = execute_recovery_next_head_finalization(&mut verify_failure, request)
        .err()
        .ok_or("verification failure was ignored")?;
    let replace_error = execute_recovery_next_head_finalization(&mut replace_failure, request)
        .err()
        .ok_or("replacement failure was ignored")?;

    assert!(matches!(
        verify_error,
        RecoveryNextHeadFinalizationError::Verify { .. }
    ));
    assert!(matches!(
        replace_error,
        RecoveryNextHeadFinalizationError::Replace { .. }
    ));
    assert_eq!(verify_failure.operations(), &[Operation::Verify(request)]);
    assert_eq!(
        replace_failure.operations(),
        &[
            Operation::Verify(request),
            Operation::SynchronizeCandidate(request),
            Operation::Replace(request),
        ]
    );
    Ok(())
}

#[test]
fn candidate_sync_failure_stops_before_head_replacement() -> Result<(), Box<dyn Error>> {
    let segment = fixture(SEGMENT_HEX)?;
    let catalog = fixture(CATALOG_ONE_HEX)?;
    let head = fixture(HEAD_ONE_HEX)?;
    let request = initial_request(&head, &catalog, &segment)?;
    let mut storage = NextHeadDouble::new(RecoveryNextHeadFinalizationReadiness::Ready)
        .fail_next_candidate_synchronization();

    let error = execute_recovery_next_head_finalization(&mut storage, request)
        .err()
        .ok_or("candidate synchronization failure was ignored")?;

    assert!(matches!(
        error,
        RecoveryNextHeadFinalizationError::SynchronizeCandidate { .. }
    ));
    assert_eq!(
        storage.operations(),
        &[
            Operation::Verify(request),
            Operation::SynchronizeCandidate(request),
        ]
    );
    Ok(())
}

#[test]
fn retry_after_replace_before_root_sync_is_already_finalized() -> Result<(), Box<dyn Error>> {
    let segment = fixture(SEGMENT_HEX)?;
    let catalog = fixture(CATALOG_ONE_HEX)?;
    let head = fixture(HEAD_ONE_HEX)?;
    let request = initial_request(&head, &catalog, &segment)?;
    let mut storage = NextHeadDouble::new(RecoveryNextHeadFinalizationReadiness::Ready)
        .fail_next_synchronization();

    let error = execute_recovery_next_head_finalization(&mut storage, request)
        .err()
        .ok_or("root synchronization failure was ignored")?;

    assert!(matches!(
        error,
        RecoveryNextHeadFinalizationError::SynchronizeRoot { .. }
    ));

    let receipt = execute_recovery_next_head_finalization(&mut storage, request)?;

    assert_eq!(
        receipt.outcome(),
        RecoveryNextHeadFinalizationOutcome::AlreadyFinalized
    );
    assert_eq!(
        storage.operations(),
        &[
            Operation::Verify(request),
            Operation::SynchronizeCandidate(request),
            Operation::Replace(request),
            Operation::SynchronizeRoot,
            Operation::Verify(request),
            Operation::SynchronizeRoot,
        ]
    );
    Ok(())
}
