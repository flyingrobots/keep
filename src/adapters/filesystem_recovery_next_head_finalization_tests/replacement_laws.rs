//! Atomic replacement and retry laws for filesystem head finalization.

use std::error::Error;
use std::fs;

use super::super::{RecoveryNextHeadFinalizationOutcome, execute_recovery_next_head_finalization};
use super::fixture::FinalizationFixture;

#[test]
fn exact_successor_atomically_replaces_the_current_head() -> Result<(), Box<dyn Error>> {
    let fixture = FinalizationFixture::new("filesystem-next-head-successor")?;
    let request = fixture.install_generation_two_candidate()?;
    let mut finalizer = fixture.finalizer()?;

    let receipt = execute_recovery_next_head_finalization(&mut finalizer, request)?;

    assert_eq!(
        receipt.outcome(),
        RecoveryNextHeadFinalizationOutcome::Finalized
    );
    assert_eq!(
        fs::read(fixture.head_path())?,
        FinalizationFixture::head_two()?
    );
    assert!(!fixture.next_head_path().exists());
    drop(finalizer);
    fixture.remove()?;
    Ok(())
}

#[test]
fn successor_retry_admits_the_exact_published_view() -> Result<(), Box<dyn Error>> {
    let fixture = FinalizationFixture::new("filesystem-next-head-successor-retry")?;
    let request = fixture.install_generation_two_candidate()?;
    let mut finalizer = fixture.finalizer()?;
    let first_receipt = execute_recovery_next_head_finalization(&mut finalizer, request)?;

    let retry = execute_recovery_next_head_finalization(&mut finalizer, request)?;

    assert_eq!(
        first_receipt.outcome(),
        RecoveryNextHeadFinalizationOutcome::Finalized
    );
    assert_eq!(
        retry.outcome(),
        RecoveryNextHeadFinalizationOutcome::AlreadyFinalized
    );
    assert_eq!(
        fs::read(fixture.head_path())?,
        FinalizationFixture::head_two()?
    );
    drop(finalizer);
    fixture.remove()?;
    Ok(())
}
