//! Filesystem next-head finalization laws.

mod fixture;
mod namespace_laws;
mod refusal_laws;
mod replacement_laws;

use std::error::Error;
use std::fs;

use super::{
    RecoveryNextHeadFinalizationOutcome, RecoveryNextHeadFinalizationReadiness,
    RecoveryNextHeadFinalizationStorage, execute_recovery_next_head_finalization,
};
use fixture::FinalizationFixture;

#[test]
fn generation_one_finalizes_and_retries_without_replacement() -> Result<(), Box<dyn Error>> {
    let fixture = FinalizationFixture::new("filesystem-next-head-initial")?;
    let request = fixture.install_generation_one_candidate()?;
    let mut finalizer = fixture.finalizer()?;

    assert_eq!(
        finalizer.verify_current(request)?,
        RecoveryNextHeadFinalizationReadiness::Ready
    );
    let receipt = execute_recovery_next_head_finalization(&mut finalizer, request)?;
    let retry = execute_recovery_next_head_finalization(&mut finalizer, request)?;

    assert_eq!(
        receipt.outcome(),
        RecoveryNextHeadFinalizationOutcome::Finalized
    );
    assert_eq!(
        retry.outcome(),
        RecoveryNextHeadFinalizationOutcome::AlreadyFinalized
    );
    assert_eq!(
        fs::read(fixture.head_path())?,
        FinalizationFixture::head_one()?
    );
    assert!(!fixture.next_head_path().exists());
    drop(finalizer);
    fixture.remove()?;
    Ok(())
}

#[test]
fn one_writer_excludes_a_second_finalizer() -> Result<(), Box<dyn Error>> {
    let fixture = FinalizationFixture::new("filesystem-next-head-writer-exclusion")?;
    let first = fixture.finalizer()?;

    let error = fixture
        .finalizer()
        .err()
        .ok_or("second next-head finalizer acquired writer authority")?;

    assert!(matches!(
        error.downcast_ref::<super::FilesystemRecoveryNextHeadFinalizationOpenError>(),
        Some(super::FilesystemRecoveryNextHeadFinalizationOpenError::WriterLock { .. })
    ));
    drop(first);
    fixture.remove()?;
    Ok(())
}
