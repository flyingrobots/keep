//! Medium filesystem laws for discard effects after a failed parent sync.
//! Oracle: successful unlink survives sync failure; absent retry creates no unlink.
//! The fault replaces the sync capability, not a kernel syscall or power loss.
//! Delete only when discard is removed or stronger runtime laws subsume these.

use std::{error::Error, fs, io};

use super::fixture::{DiscardFixture, request, truncated_bytes};
use crate::adapters::{
    FilesystemRecoveryStageDiscarder, RecoveryStage, RecoveryStageDiscardOutcome,
    RecoveryStageDiscardStorage, RecoveryStageDiscardStorageError, RecoveryStageEvidence,
    RecoveryStageParent, execute_recovery_stage_discard,
};

#[test]
fn failed_parent_sync_reports_the_completed_removal() -> Result<(), Box<dyn Error>> {
    check_failure(RecoveryStageDiscardOutcome::Removed)
}

#[test]
fn failed_parent_sync_reports_an_already_absent_retry() -> Result<(), Box<dyn Error>> {
    check_failure(RecoveryStageDiscardOutcome::AlreadyAbsent)
}

fn check_failure(outcome: RecoveryStageDiscardOutcome) -> Result<(), Box<dyn Error>> {
    let fixture = DiscardFixture::new(&format!("discard-sync-{outcome:?}"))?;
    let stage = RecoveryStage::NextHead;
    let bytes = truncated_bytes(stage, 1)?;
    let request = request(stage, &bytes)?;
    fs::write(fixture.stage_path(stage), &bytes)?;
    let mut storage = FailParentSync(fixture.discarder()?);
    if outcome == RecoveryStageDiscardOutcome::AlreadyAbsent {
        let _receipt = execute_recovery_stage_discard(&mut storage.0, request)?;
    }

    let error = execute_recovery_stage_discard(&mut storage, request)
        .err()
        .ok_or("parent sync failure returned a durable receipt")?;

    assert!(
        !fixture.stage_path(stage).try_exists()?,
        "unlink was not rolled back"
    );
    let cause = error
        .source()
        .and_then(|cause| cause.downcast_ref::<io::Error>())
        .ok_or("original synchronization cause was lost")?;
    assert_eq!(cause.raw_os_error(), Some(5), "original EIO remains typed");
    assert_eq!(
        error.to_string(),
        format!(
            "{stage} parent synchronization failed after {outcome:?} (durability unconfirmed): {cause}"
        ),
        "failure must distinguish this call's removal from an absent retry"
    );
    drop(storage);
    let mut reopened = fixture.discarder()?;
    let receipt = execute_recovery_stage_discard(&mut reopened, request)?;
    assert_eq!(
        receipt.outcome(),
        RecoveryStageDiscardOutcome::AlreadyAbsent
    );
    drop(reopened);
    fixture.remove()?;
    Ok(())
}

struct FailParentSync(FilesystemRecoveryStageDiscarder);

impl RecoveryStageDiscardStorage for FailParentSync {
    fn remove_if_matching(
        &mut self,
        evidence: RecoveryStageEvidence,
    ) -> Result<RecoveryStageDiscardOutcome, RecoveryStageDiscardStorageError> {
        self.0.remove_if_matching(evidence)
    }

    fn synchronize_parent(&mut self, _parent: RecoveryStageParent) -> io::Result<()> {
        Err(io::Error::from_raw_os_error(5))
    }
}
