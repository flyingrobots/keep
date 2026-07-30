//! Pinned-filesystem reusable-segment continuation laws.

use std::error::Error;
use std::fs;

use super::{
    AdmittedSegment, AdmittedSegmentRecord, FilesystemRecoverySegmentResumeOpenError,
    WriterLockAcquireError, execute_recovery_segment_resume,
};

mod fixture;
mod refusal_laws;

use fixture::{ResumeFixture, empty_prefix, maximum_policy, resume_request, reusable_prefix};

#[test]
fn empty_prefix_resumes_and_seals_as_an_empty_segment() -> Result<(), Box<dyn Error>> {
    let fixture = ResumeFixture::new("filesystem-empty-segment-resume")?;
    let prefix = empty_prefix()?;
    fs::write(fixture.stage_path(), &prefix)?;

    let resumed = execute_recovery_segment_resume(fixture.resumer()?, resume_request(&prefix)?)?;
    let sealed = resumed.seal()?;
    assert_eq!(sealed.record_count(), 0);
    let _closed = sealed.close();

    let observed = fs::read(fixture.stage_path())?;
    let admitted = AdmittedSegment::decode(&observed, maximum_policy())?;
    assert!(observed.starts_with(&prefix));
    assert_eq!(admitted.record_count(), 0);
    fixture.remove()?;
    Ok(())
}

#[test]
fn exact_prefix_resumes_seals_and_retains_writer_authority() -> Result<(), Box<dyn Error>> {
    let fixture = ResumeFixture::new("filesystem-segment-resume")?;
    let prefix = reusable_prefix()?;
    fs::write(fixture.stage_path(), &prefix)?;
    let authority = fixture.resumer()?;

    let resumed_stage = execute_recovery_segment_resume(authority, resume_request(&prefix)?)?;
    let second = fixture
        .resumer()
        .err()
        .ok_or("resumed stage did not retain writer authority")?;
    let sealed = resumed_stage
        .append(AdmittedSegmentRecord::for_chunk(&[1])?)?
        .seal()?;

    assert!(matches!(
        second,
        FilesystemRecoverySegmentResumeOpenError::WriterLock {
            source: WriterLockAcquireError::Busy
        }
    ));
    assert_eq!(sealed.record_count(), 2);
    let _closed = sealed.close();

    let observed = fs::read(fixture.stage_path())?;
    let admitted = AdmittedSegment::decode(&observed, maximum_policy())?;
    assert!(observed.starts_with(&prefix));
    assert_eq!(admitted.record_count(), 2);
    drop(fixture.resumer()?);
    fixture.remove()?;
    Ok(())
}
