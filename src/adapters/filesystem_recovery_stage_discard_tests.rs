//! Pinned-filesystem truncated-stage discard laws.

use std::error::Error;
use std::fs;

use super::{
    FilesystemRecoveryStageDiscardOpenError, FilesystemRecoveryStageError, RecoveryStage,
    RecoveryStageDiscardError, RecoveryStageDiscardOutcome, RecoveryStageDiscardStorageError,
    WriterLockAcquireError, execute_recovery_stage_discard,
};

mod fixture;

use fixture::{DiscardFixture, evidence, request, truncated_bytes};

#[test]
fn exact_stage_discard_and_absent_retry_are_durable_for_every_parent() -> Result<(), Box<dyn Error>>
{
    let fixture = DiscardFixture::new("filesystem-stage-discard")?;
    let stages = [
        RecoveryStage::Segment,
        RecoveryStage::Catalog,
        RecoveryStage::NextHead,
    ];
    for stage in stages {
        fs::write(fixture.stage_path(stage), truncated_bytes(stage, 1)?)?;
    }
    let mut discarder = fixture.discarder()?;

    for stage in stages {
        let bytes = truncated_bytes(stage, 1)?;
        let request = request(stage, &bytes)?;
        let removed = execute_recovery_stage_discard(&mut discarder, request)?;
        let retried = execute_recovery_stage_discard(&mut discarder, request)?;

        assert_eq!(removed.outcome(), RecoveryStageDiscardOutcome::Removed);
        assert_eq!(
            retried.outcome(),
            RecoveryStageDiscardOutcome::AlreadyAbsent
        );
        assert!(!fixture.stage_path(stage).exists());
    }
    drop(discarder);
    fixture.remove()?;
    Ok(())
}

#[test]
fn changed_stage_evidence_is_preserved_and_refused_before_unlink() -> Result<(), Box<dyn Error>> {
    let fixture = DiscardFixture::new("filesystem-stage-discard-mismatch")?;
    let old = truncated_bytes(RecoveryStage::Segment, 1)?;
    let new = truncated_bytes(RecoveryStage::Segment, 2)?;
    let expected = request(RecoveryStage::Segment, &old)?;
    fs::write(fixture.stage_path(RecoveryStage::Segment), &new)?;
    let observed = evidence(RecoveryStage::Segment, &new)?;
    let mut discarder = fixture.discarder()?;

    let error = execute_recovery_stage_discard(&mut discarder, expected)
        .err()
        .ok_or("changed recovery stage was removed")?;

    assert!(matches!(
        error,
        RecoveryStageDiscardError::Remove {
            source: RecoveryStageDiscardStorageError::EvidenceMismatch {
                expected: actual_expected,
                observed: actual_observed,
            },
        } if actual_expected == expected.evidence() && actual_observed == observed
    ));
    assert_eq!(fs::read(fixture.stage_path(RecoveryStage::Segment))?, new);
    drop(discarder);
    fixture.remove()?;
    Ok(())
}

#[test]
fn symbolic_stage_is_never_followed_or_removed() -> Result<(), Box<dyn Error>> {
    use std::os::unix::fs::symlink;

    let fixture = DiscardFixture::new("filesystem-stage-discard-symlink")?;
    let bytes = truncated_bytes(RecoveryStage::Segment, 1)?;
    let request = request(RecoveryStage::Segment, &bytes)?;
    let target = fixture.root().join("outside");
    fs::write(&target, &bytes)?;
    symlink(&target, fixture.stage_path(RecoveryStage::Segment))?;
    let mut discarder = fixture.discarder()?;

    let error = execute_recovery_stage_discard(&mut discarder, request)
        .err()
        .ok_or("symbolic recovery stage was followed")?;

    let RecoveryStageDiscardError::Remove { source } = error else {
        return Err("symbolic-stage refusal lost its removal phase".into());
    };
    assert!(matches!(
        filesystem_stage_source(&source)?,
        FilesystemRecoveryStageError::Open {
            stage: RecoveryStage::Segment,
            ..
        }
    ));
    assert_eq!(fs::read(&target)?, bytes);
    assert!(fixture.stage_path(RecoveryStage::Segment).is_symlink());
    drop(discarder);
    fixture.remove()?;
    Ok(())
}

#[test]
fn replacement_after_open_refuses_without_removing_the_new_entry() -> Result<(), Box<dyn Error>> {
    let fixture = DiscardFixture::new("filesystem-stage-discard-replaced")?;
    let stage_path = fixture.stage_path(RecoveryStage::Segment);
    let retained_path = fixture.root().join("retained-stage");
    let old = truncated_bytes(RecoveryStage::Segment, 1)?;
    let new = truncated_bytes(RecoveryStage::Segment, 2)?;
    fs::write(&stage_path, &old)?;
    let request = request(RecoveryStage::Segment, &old)?;
    let discarder = fixture.discarder()?;
    let mut hook_result = Ok(());

    let result = discarder.remove_if_matching_with(request.evidence(), || {
        hook_result =
            fs::rename(&stage_path, &retained_path).and_then(|()| fs::write(&stage_path, &new));
    });

    hook_result?;
    let error = result.err().ok_or("replaced recovery stage was removed")?;
    assert!(matches!(
        filesystem_stage_source(&error)?,
        FilesystemRecoveryStageError::Replaced {
            stage: RecoveryStage::Segment,
        }
    ));
    assert_eq!(fs::read(&stage_path)?, new);
    assert_eq!(fs::read(&retained_path)?, old);
    drop(discarder);
    fixture.remove()?;
    Ok(())
}

#[test]
fn replacement_after_observation_refuses_without_removing_the_new_entry()
-> Result<(), Box<dyn Error>> {
    let fixture = DiscardFixture::new("filesystem-stage-discard-final-handoff")?;
    let stage_path = fixture.stage_path(RecoveryStage::Segment);
    let retained_path = fixture.root().join("retained-observed-stage");
    let old = truncated_bytes(RecoveryStage::Segment, 1)?;
    let new = truncated_bytes(RecoveryStage::Segment, 2)?;
    fs::write(&stage_path, &old)?;
    let request = request(RecoveryStage::Segment, &old)?;
    let discarder = fixture.discarder()?;
    let mut hook_result = Ok(());

    let result = discarder.remove_if_matching_after_observation_with(request.evidence(), || {
        hook_result =
            fs::rename(&stage_path, &retained_path).and_then(|()| fs::write(&stage_path, &new));
    });

    hook_result?;
    let error = result.err().ok_or("replacement at handoff was removed")?;
    assert!(matches!(
        filesystem_stage_source(&error)?,
        FilesystemRecoveryStageError::Replaced {
            stage: RecoveryStage::Segment,
        }
    ));
    assert_eq!(fs::read(&stage_path)?, new);
    assert_eq!(fs::read(&retained_path)?, old);
    drop(discarder);
    fixture.remove()?;
    Ok(())
}

fn filesystem_stage_source(
    error: &RecoveryStageDiscardStorageError,
) -> Result<&FilesystemRecoveryStageError, &'static str> {
    let RecoveryStageDiscardStorageError::Storage { source } = error else {
        return Err("recovery-stage failure lost its storage boundary");
    };
    source
        .get_ref()
        .and_then(|source| source.downcast_ref::<FilesystemRecoveryStageError>())
        .ok_or("recovery-stage storage failure lost its typed source")
}

#[test]
fn retained_discarder_authority_excludes_a_second_writer() -> Result<(), Box<dyn Error>> {
    let fixture = DiscardFixture::new("filesystem-stage-discard-lock")?;
    let first = fixture.discarder()?;

    let error = fixture
        .discarder()
        .err()
        .ok_or("second recovery writer acquired authority")?;

    assert!(matches!(
        error,
        FilesystemRecoveryStageDiscardOpenError::WriterLock {
            source: WriterLockAcquireError::Busy,
        }
    ));
    drop(first);
    fixture.remove()?;
    Ok(())
}
