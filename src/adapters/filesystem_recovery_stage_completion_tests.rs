//! Pinned-filesystem complete-stage recovery laws.

use std::error::Error;
use std::fs;

use super::{
    FilesystemRecoveryStageCompletionOpenError, RecoveryStage, RecoveryStageDiscardOutcome,
    RecoveryStagePoolOutcome, RecoveryStageSynchronizationOutcome, WriterLockAcquireError,
    execute_recovery_stage_completion,
};

mod fixture;
mod refusal_laws;
mod replacement_laws;

use fixture::{CompletionFixture, catalog_bytes, request, segment_bytes};

#[test]
fn exact_segment_and_catalog_complete_to_durable_orphans() -> Result<(), Box<dyn Error>> {
    let fixture = CompletionFixture::new("filesystem-stage-completion")?;
    let cases = [
        (RecoveryStage::Segment, segment_bytes()?),
        (RecoveryStage::Catalog, catalog_bytes()?),
    ];
    let mut completer = fixture.completer()?;

    for (stage, bytes) in cases {
        let request = request(stage, &bytes)?;
        fs::write(fixture.stage_path(stage), &bytes)?;

        let completed = execute_recovery_stage_completion(&mut completer, request)?;
        let retried = execute_recovery_stage_completion(&mut completer, request)?;
        fs::write(fixture.stage_path(stage), &bytes)?;
        let reappeared = execute_recovery_stage_completion(&mut completer, request)?;

        assert_eq!(
            completed.synchronization_outcome(),
            RecoveryStageSynchronizationOutcome::Synchronized
        );
        assert_eq!(completed.pool_outcome(), RecoveryStagePoolOutcome::Linked);
        assert_eq!(
            completed.stage_outcome(),
            RecoveryStageDiscardOutcome::Removed
        );
        assert_eq!(
            retried.synchronization_outcome(),
            RecoveryStageSynchronizationOutcome::AlreadyAbsent
        );
        assert_eq!(
            retried.pool_outcome(),
            RecoveryStagePoolOutcome::AlreadyPresent
        );
        assert_eq!(
            retried.stage_outcome(),
            RecoveryStageDiscardOutcome::AlreadyAbsent
        );
        assert_eq!(
            reappeared.synchronization_outcome(),
            RecoveryStageSynchronizationOutcome::Synchronized
        );
        assert_eq!(
            reappeared.pool_outcome(),
            RecoveryStagePoolOutcome::AlreadyPresent
        );
        assert_eq!(
            reappeared.stage_outcome(),
            RecoveryStageDiscardOutcome::Removed
        );
        assert_eq!(fs::read(fixture.pool_path(request))?, bytes);
        assert!(!fixture.stage_path(stage).exists());
    }
    drop(completer);
    fixture.remove()?;
    Ok(())
}

#[test]
fn retained_completer_authority_excludes_a_second_writer() -> Result<(), Box<dyn Error>> {
    let fixture = CompletionFixture::new("filesystem-stage-completion-lock")?;
    let first = fixture.completer()?;

    let error = fixture
        .completer()
        .err()
        .ok_or("second recovery writer acquired authority")?;

    assert!(matches!(
        error,
        FilesystemRecoveryStageCompletionOpenError::WriterLock {
            source: WriterLockAcquireError::Busy,
        }
    ));
    drop(first);
    fixture.remove()?;
    Ok(())
}
