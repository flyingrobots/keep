//! Filesystem next-head evidence and namespace refusal laws.

use std::error::Error;
use std::fs;

use super::super::{
    FilesystemRecoveryStageError, RecoveryNextHeadFinalizationError,
    RecoveryNextHeadFinalizationReadiness, RecoveryNextHeadFinalizationStorage,
    RecoveryNextHeadFinalizationStorageError, execute_recovery_next_head_finalization,
};
use super::fixture::FinalizationFixture;

#[test]
fn symbolic_candidate_is_never_followed_or_published() -> Result<(), Box<dyn Error>> {
    use std::os::unix::fs::symlink;

    let fixture = FinalizationFixture::new("filesystem-next-head-link")?;
    let request = fixture.install_generation_one_candidate()?;
    let target = fixture.root().join("outside-head");
    fs::rename(fixture.next_head_path(), &target)?;
    symlink(&target, fixture.next_head_path())?;
    let mut finalizer = fixture.finalizer()?;

    let error = execute_recovery_next_head_finalization(&mut finalizer, request)
        .err()
        .ok_or("symbolic next head was followed")?;

    let RecoveryNextHeadFinalizationError::Verify { source, .. } = error else {
        return Err("symbolic candidate refusal lost the verification phase".into());
    };
    let RecoveryNextHeadFinalizationStorageError::Stage { source } = source.as_ref() else {
        return Err("symbolic candidate refusal lost the stage boundary".into());
    };
    assert!(matches!(
        source.as_ref(),
        FilesystemRecoveryStageError::Open { .. }
    ));
    assert!(!fixture.head_path().exists());
    assert!(fixture.next_head_path().is_symlink());
    drop(finalizer);
    fixture.remove()?;
    Ok(())
}

#[test]
fn changed_candidate_is_refused_at_the_replacement_boundary() -> Result<(), Box<dyn Error>> {
    let fixture = FinalizationFixture::new("filesystem-next-head-replacement-drift")?;
    let request = fixture.install_generation_one_candidate()?;
    let mut finalizer = fixture.finalizer()?;
    assert_eq!(
        finalizer.verify_current(request)?,
        RecoveryNextHeadFinalizationReadiness::Ready
    );
    fs::write(fixture.next_head_path(), [0_u8])?;

    let error = finalizer
        .replace_head(request)
        .err()
        .ok_or("changed next head replaced durable HEAD")?;

    assert!(matches!(
        error,
        RecoveryNextHeadFinalizationStorageError::EvidenceMismatch { .. }
    ));
    assert!(!fixture.head_path().exists());
    assert_eq!(fs::read(fixture.next_head_path())?, [0_u8]);
    drop(finalizer);
    fixture.remove()?;
    Ok(())
}

#[test]
fn disappeared_current_head_refuses_successor_without_mutation() -> Result<(), Box<dyn Error>> {
    let fixture = FinalizationFixture::new("filesystem-next-head-current-missing")?;
    let request = fixture.install_generation_two_candidate()?;
    fs::remove_file(fixture.head_path())?;
    let mut finalizer = fixture.finalizer()?;

    let error = execute_recovery_next_head_finalization(&mut finalizer, request)
        .err()
        .ok_or("successor finalized after current HEAD disappeared")?;

    assert!(matches!(
        error,
        RecoveryNextHeadFinalizationError::Verify { source, .. }
            if matches!(
                source.as_ref(),
                RecoveryNextHeadFinalizationStorageError::CurrentMismatch {
                    observed: None,
                    ..
                }
            )
    ));
    assert_eq!(
        fs::read(fixture.next_head_path())?,
        FinalizationFixture::head_two()?
    );
    assert!(!fixture.head_path().exists());
    drop(finalizer);
    fixture.remove()?;
    Ok(())
}

#[test]
fn corrupt_candidate_catalog_refuses_before_head_replacement() -> Result<(), Box<dyn Error>> {
    let fixture = FinalizationFixture::new("filesystem-next-head-catalog-corrupt")?;
    let request = fixture.install_generation_one_candidate()?;
    fs::write(fixture.catalog_one_path()?, b"corrupt")?;
    let mut finalizer = fixture.finalizer()?;

    let error = execute_recovery_next_head_finalization(&mut finalizer, request)
        .err()
        .ok_or("candidate with corrupt catalog was finalized")?;

    assert!(matches!(
        error,
        RecoveryNextHeadFinalizationError::Verify { source, .. }
            if matches!(
                source.as_ref(),
                RecoveryNextHeadFinalizationStorageError::CandidateView { .. }
            )
    ));
    assert!(!fixture.head_path().exists());
    assert_eq!(
        fs::read(fixture.next_head_path())?,
        FinalizationFixture::head_one()?
    );
    drop(finalizer);
    fixture.remove()?;
    Ok(())
}

#[test]
fn already_finalized_retry_refuses_a_reappeared_candidate() -> Result<(), Box<dyn Error>> {
    let fixture = FinalizationFixture::new("filesystem-next-head-reappeared")?;
    let request = fixture.install_generation_one_candidate()?;
    let mut finalizer = fixture.finalizer()?;
    let first_receipt = execute_recovery_next_head_finalization(&mut finalizer, request)?;
    fs::write(fixture.next_head_path(), FinalizationFixture::head_one()?)?;

    let error = execute_recovery_next_head_finalization(&mut finalizer, request)
        .err()
        .ok_or("already-finalized retry ignored a reappeared candidate")?;

    assert_eq!(
        first_receipt.outcome(),
        super::super::RecoveryNextHeadFinalizationOutcome::Finalized
    );
    assert!(matches!(
        error,
        RecoveryNextHeadFinalizationError::Verify { source, .. }
            if matches!(
                source.as_ref(),
                RecoveryNextHeadFinalizationStorageError::UnexpectedCandidate { .. }
            )
    ));
    assert_eq!(
        fs::read(fixture.head_path())?,
        FinalizationFixture::head_one()?
    );
    assert_eq!(
        fs::read(fixture.next_head_path())?,
        FinalizationFixture::head_one()?
    );
    drop(finalizer);
    fixture.remove()?;
    Ok(())
}

#[test]
fn absent_candidate_reports_typed_missing_without_creating_head() -> Result<(), Box<dyn Error>> {
    let fixture = FinalizationFixture::new("filesystem-next-head-missing")?;
    let request = fixture.install_generation_one_candidate()?;
    fs::remove_file(fixture.next_head_path())?;
    let mut finalizer = fixture.finalizer()?;

    let error = execute_recovery_next_head_finalization(&mut finalizer, request)
        .err()
        .ok_or("absent candidate produced a finalization receipt")?;

    assert!(matches!(
        error,
        RecoveryNextHeadFinalizationError::Verify { source, .. }
            if matches!(
                source.as_ref(),
                RecoveryNextHeadFinalizationStorageError::MissingCandidate {
                    expected,
                } if *expected == request.evidence()
            )
    ));
    assert!(!fixture.head_path().exists());
    assert!(!fixture.next_head_path().exists());
    drop(finalizer);
    fixture.remove()?;
    Ok(())
}
