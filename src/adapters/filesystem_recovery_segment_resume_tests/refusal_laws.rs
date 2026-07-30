//! Filesystem continuation evidence and namespace refusal laws.

use std::error::Error;
use std::fs;
use std::os::unix::fs::symlink;

use super::super::{
    FilesystemRecoveryStageError, RecoverySegmentResumeError, RecoverySegmentResumeStorageError,
    RecoveryStage, RecoveryStageNamespacePhase, execute_recovery_segment_resume,
};
use super::{ResumeFixture, resume_request, reusable_prefix};

#[test]
fn absent_stage_is_a_typed_refusal() -> Result<(), Box<dyn Error>> {
    let fixture = ResumeFixture::new("filesystem-segment-resume-missing")?;
    let prefix = reusable_prefix()?;

    let error = execute_recovery_segment_resume(fixture.resumer()?, resume_request(&prefix)?)
        .err()
        .ok_or("absent stage unexpectedly resumed")?;

    assert!(matches!(
        error,
        RecoverySegmentResumeError::Open {
            source: RecoverySegmentResumeStorageError::Missing { .. }
        }
    ));
    fixture.remove()?;
    Ok(())
}

#[test]
fn changed_stage_is_refused_before_a_write() -> Result<(), Box<dyn Error>> {
    let fixture = ResumeFixture::new("filesystem-segment-resume-changed")?;
    let prefix = reusable_prefix()?;
    let request = resume_request(&prefix)?;
    let mut changed = prefix;
    let tail = changed.last_mut().ok_or("missing segment tail")?;
    *tail ^= 1;
    fs::write(fixture.stage_path(), &changed)?;

    let error = execute_recovery_segment_resume(fixture.resumer()?, request)
        .err()
        .ok_or("changed stage unexpectedly resumed")?;

    assert!(matches!(
        error,
        RecoverySegmentResumeError::Open {
            source: RecoverySegmentResumeStorageError::EvidenceMismatch { .. }
        }
    ));
    assert_eq!(fs::read(fixture.stage_path())?, changed);
    fixture.remove()?;
    Ok(())
}

#[test]
fn symbolic_stage_is_never_followed() -> Result<(), Box<dyn Error>> {
    let fixture = ResumeFixture::new("filesystem-segment-resume-link")?;
    let prefix = reusable_prefix()?;
    let target = fixture.root().join("target.seg");
    fs::write(&target, &prefix)?;
    symlink(&target, fixture.stage_path())?;

    let error = execute_recovery_segment_resume(fixture.resumer()?, resume_request(&prefix)?)
        .err()
        .ok_or("symbolic stage unexpectedly resumed")?;

    assert!(stage_error_matches(&error, |source| {
        matches!(
            source,
            FilesystemRecoveryStageError::Open {
                stage: RecoveryStage::Segment,
                ..
            }
        )
    }));
    assert_eq!(fs::read(target)?, prefix);
    fixture.remove()?;
    Ok(())
}

#[test]
fn replacement_at_the_writable_handoff_is_preserved_and_refused() -> Result<(), Box<dyn Error>> {
    let fixture = ResumeFixture::new("filesystem-segment-resume-replaced")?;
    let prefix = reusable_prefix()?;
    fs::write(fixture.stage_path(), &prefix)?;
    let stage_path = fixture.stage_path();
    let retained = fixture.root().join("retained.seg");
    let replacement = b"replacement".to_vec();
    let replacement_for_hook = replacement.clone();
    let stage_for_hook = stage_path.clone();
    let resumer = fixture.resumer_before_handoff(move || {
        let _renamed = fs::rename(&stage_for_hook, &retained);
        let _written = fs::write(&stage_for_hook, replacement_for_hook);
    })?;

    let error = execute_recovery_segment_resume(resumer, resume_request(&prefix)?)
        .err()
        .ok_or("replaced stage unexpectedly resumed")?;

    assert!(stage_error_matches(&error, |source| {
        matches!(
            source,
            FilesystemRecoveryStageError::Replaced {
                stage: RecoveryStage::Segment
            }
        )
    }));
    assert_eq!(fs::read(stage_path)?, replacement);
    fixture.remove()?;
    Ok(())
}

#[test]
fn byte_replacement_at_the_writable_handoff_is_preserved_and_refused() -> Result<(), Box<dyn Error>>
{
    let fixture = ResumeFixture::new("filesystem-segment-resume-byte-replaced")?;
    let prefix = reusable_prefix()?;
    fs::write(fixture.stage_path(), &prefix)?;
    let stage_path = fixture.stage_path();
    let mut replacement = prefix.clone();
    let tail = replacement.last_mut().ok_or("missing segment tail")?;
    *tail ^= 1;
    let replacement_for_hook = replacement.clone();
    let stage_for_hook = stage_path.clone();
    let resumer = fixture.resumer_before_handoff(move || {
        let _written = fs::write(&stage_for_hook, replacement_for_hook);
    })?;

    let error = execute_recovery_segment_resume(resumer, resume_request(&prefix)?)
        .err()
        .ok_or("byte-replaced stage unexpectedly resumed")?;

    assert!(matches!(
        error,
        RecoverySegmentResumeError::Open {
            source: RecoverySegmentResumeStorageError::EvidenceMismatch { .. }
        }
    ));
    assert_eq!(fs::read(stage_path)?, replacement);
    fixture.remove()?;
    Ok(())
}

#[test]
fn replaced_staging_namespace_is_refused_before_stage_open() -> Result<(), Box<dyn Error>> {
    let fixture = ResumeFixture::new("filesystem-segment-resume-namespace")?;
    let prefix = reusable_prefix()?;
    fs::write(fixture.stage_path(), &prefix)?;
    let resumer = fixture.resumer()?;
    fs::rename(
        fixture.root().join("staging"),
        fixture.root().join("old-staging"),
    )?;
    fs::create_dir(fixture.root().join("staging"))?;
    fs::write(fixture.stage_path(), &prefix)?;

    let error = execute_recovery_segment_resume(resumer, resume_request(&prefix)?)
        .err()
        .ok_or("replaced namespace unexpectedly resumed")?;

    assert!(stage_error_matches(&error, |source| {
        matches!(
            source,
            FilesystemRecoveryStageError::Namespace {
                stage: RecoveryStage::Segment,
                phase: RecoveryStageNamespacePhase::BeforeObservation,
                ..
            }
        )
    }));
    fixture.remove()?;
    Ok(())
}

fn stage_error_matches(
    error: &RecoverySegmentResumeError,
    predicate: impl FnOnce(&FilesystemRecoveryStageError) -> bool,
) -> bool {
    let RecoverySegmentResumeError::Open {
        source: RecoverySegmentResumeStorageError::Storage { source },
    } = error
    else {
        return false;
    };
    let Some(source) = source
        .get_ref()
        .and_then(|source| source.downcast_ref::<FilesystemRecoveryStageError>())
    else {
        return false;
    };
    predicate(source)
}
