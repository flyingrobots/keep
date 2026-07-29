//! Reusable-segment recovery execution laws.

use std::error::Error;

use keep::{
    AdmittedSegment, AdmittedSegmentRecord, RecoverySegmentResumeError, SegmentWriteError,
    execute_recovery_segment_resume,
};

use super::storage_double::MemoryResumeStorage;
use super::support::require_error;
use super::{maximum_policy, resume_request, reusable_prefix};

#[test]
fn resumed_prefix_appends_and_seals_without_rewriting_admitted_bytes() -> Result<(), Box<dyn Error>>
{
    let prefix = reusable_prefix()?;
    let storage = MemoryResumeStorage::available(&prefix);
    let probe = storage.probe();

    let resumed = execute_recovery_segment_resume(storage, resume_request(&prefix)?)?;
    let sealed = resumed
        .append(AdmittedSegmentRecord::for_chunk(&[1])?)?
        .seal()?;
    let observed = probe.borrow().clone();
    let admitted = AdmittedSegment::decode(&observed, maximum_policy())?;

    assert!(observed.starts_with(&prefix));
    assert_eq!(sealed.record_count(), 2);
    assert_eq!(admitted.record_count(), 2);
    Ok(())
}

#[test]
fn resumed_prefix_retains_duplicate_identity_refusal() -> Result<(), Box<dyn Error>> {
    let prefix = reusable_prefix()?;
    let storage = MemoryResumeStorage::available(&prefix);
    let resumed = execute_recovery_segment_resume(storage, resume_request(&prefix)?)?;
    let duplicate = AdmittedSegmentRecord::for_chunk(&[0])?;

    let error = require_error(
        resumed.append(duplicate),
        "resumed prefix must retain prior identities",
    )?;

    assert!(matches!(
        error,
        SegmentWriteError::DuplicateRecordIdentity { identity }
            if identity == duplicate.identity()
    ));
    Ok(())
}

#[test]
fn changed_materialized_bytes_are_refused_before_a_write() -> Result<(), Box<dyn Error>> {
    let prefix = reusable_prefix()?;
    let mut changed = prefix.clone();
    let tail = changed.last_mut().ok_or("missing prefix tail")?;
    *tail ^= 1;
    let storage = MemoryResumeStorage::available(&changed);
    let probe = storage.probe();

    let error = require_error(
        execute_recovery_segment_resume(storage, resume_request(&prefix)?),
        "changed bytes must not be resumed",
    )?;

    assert!(matches!(
        error,
        RecoverySegmentResumeError::Admission { .. }
    ));
    assert_eq!(*probe.borrow(), changed);
    Ok(())
}

#[test]
fn storage_failure_returns_no_resumable_stage() -> Result<(), Box<dyn Error>> {
    let prefix = reusable_prefix()?;
    let storage = MemoryResumeStorage::failing(&prefix);

    let error = require_error(
        execute_recovery_segment_resume(storage, resume_request(&prefix)?),
        "storage failure must prevent continuation",
    )?;

    assert!(matches!(error, RecoverySegmentResumeError::Open { .. }));
    Ok(())
}
