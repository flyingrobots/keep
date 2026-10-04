//! Medium filesystem laws for publication effects after a failed root sync.
//! Oracle: replacement survives sync failure; an already-current retry does not replace.
//! The fault replaces the sync capability, not a kernel syscall or power loss.
//! Delete only when finalization is removed or stronger runtime laws subsume these.

use std::{error::Error, fs, io};

use super::fixture::FinalizationFixture;
use crate::adapters::{
    FilesystemRecoveryNextHeadFinalizer, RecoveryNextHeadFinalizationError,
    RecoveryNextHeadFinalizationOutcome, RecoveryNextHeadFinalizationReadiness,
    RecoveryNextHeadFinalizationRequest, RecoveryNextHeadFinalizationStorage,
    RecoveryNextHeadFinalizationStorageError, execute_recovery_next_head_finalization,
};

#[test]
fn failed_root_sync_reports_the_completed_replacement() -> Result<(), Box<dyn Error>> {
    check_failure(RecoveryNextHeadFinalizationOutcome::Finalized)
}

#[test]
fn failed_root_sync_reports_an_already_current_retry() -> Result<(), Box<dyn Error>> {
    check_failure(RecoveryNextHeadFinalizationOutcome::AlreadyFinalized)
}

fn check_failure(outcome: RecoveryNextHeadFinalizationOutcome) -> Result<(), Box<dyn Error>> {
    let fixture = FinalizationFixture::new(&format!("finalization-sync-{outcome:?}"))?;
    let request = fixture.install_generation_two_candidate()?;
    let mut storage = FailRootSync(fixture.finalizer()?);
    if outcome == RecoveryNextHeadFinalizationOutcome::AlreadyFinalized {
        let _receipt = execute_recovery_next_head_finalization(&mut storage.0, request)?;
    }

    let error = execute_recovery_next_head_finalization(&mut storage, request)
        .err()
        .ok_or("root sync failure returned a durable receipt")?;

    assert_eq!(
        fs::read(fixture.head_path())?,
        FinalizationFixture::head_two()?,
        "replacement was not rolled back"
    );
    assert!(
        !fixture.next_head_path().try_exists()?,
        "rename consumed the stage name"
    );
    let cause = error
        .source()
        .and_then(|cause| cause.downcast_ref::<io::Error>())
        .ok_or("original synchronization cause was lost")?;
    assert_eq!(cause.raw_os_error(), Some(5), "original EIO remains typed");
    assert!(
        matches!(&error, RecoveryNextHeadFinalizationError::SynchronizeRoot { target, outcome: actual, .. }
        if *target == request.target() && *actual == outcome),
        "typed publication effects must remain available"
    );
    assert_eq!(
        error.to_string(),
        format!(
            "failed to synchronize recovery head generation {} after {outcome:?} (durability unconfirmed): {cause}",
            request.target().generation().get()
        ),
        "failure must distinguish this call's replacement from an already-current retry"
    );
    drop(storage);
    let mut reopened = fixture.finalizer()?;
    let receipt = execute_recovery_next_head_finalization(&mut reopened, request)?;
    assert_eq!(
        receipt.outcome(),
        RecoveryNextHeadFinalizationOutcome::AlreadyFinalized
    );
    drop(reopened);
    fixture.remove()?;
    Ok(())
}

struct FailRootSync(FilesystemRecoveryNextHeadFinalizer);

impl RecoveryNextHeadFinalizationStorage for FailRootSync {
    fn verify_current(
        &mut self,
        request: RecoveryNextHeadFinalizationRequest,
    ) -> Result<RecoveryNextHeadFinalizationReadiness, RecoveryNextHeadFinalizationStorageError>
    {
        self.0.verify_current(request)
    }

    fn synchronize_candidate(
        &mut self,
        request: RecoveryNextHeadFinalizationRequest,
    ) -> Result<(), RecoveryNextHeadFinalizationStorageError> {
        self.0.synchronize_candidate(request)
    }

    fn replace_head(
        &mut self,
        request: RecoveryNextHeadFinalizationRequest,
    ) -> Result<(), RecoveryNextHeadFinalizationStorageError> {
        self.0.replace_head(request)
    }

    fn synchronize_root(&mut self) -> io::Result<()> {
        Err(io::Error::from_raw_os_error(5))
    }
}
