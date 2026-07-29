//! Filesystem complete-stage refusal and replacement laws.

use std::error::Error;
use std::fs;

use super::super::{
    FilesystemRecoveryStageError, RecoveryStage, RecoveryStageCompletionError,
    RecoveryStageCompletionStorage, RecoveryStageCompletionStorageError,
    execute_recovery_stage_completion,
};
use super::fixture::{CompletionFixture, catalog_bytes, request, segment_bytes};

#[test]
fn conflicting_pool_is_refused_without_removing_the_exact_stage() -> Result<(), Box<dyn Error>> {
    let fixture = CompletionFixture::new("filesystem-stage-completion-conflict")?;
    let bytes = segment_bytes()?;
    let request = request(RecoveryStage::Segment, &bytes)?;
    fs::write(fixture.stage_path(RecoveryStage::Segment), &bytes)?;
    fs::write(fixture.pool_path(request), b"conflict")?;
    let mut completer = fixture.completer()?;

    let error = execute_recovery_stage_completion(&mut completer, request)
        .err()
        .ok_or("conflicting pool artifact was accepted")?;

    assert!(matches!(
        error,
        RecoveryStageCompletionError::VerifyPool { target, .. }
            if target == request.target()
    ));
    assert_eq!(fs::read(fixture.stage_path(RecoveryStage::Segment))?, bytes);
    assert_eq!(fs::read(fixture.pool_path(request))?, b"conflict");
    drop(completer);
    fixture.remove()?;
    Ok(())
}

#[test]
fn symbolic_stage_is_never_followed_or_linked() -> Result<(), Box<dyn Error>> {
    use std::os::unix::fs::symlink;

    let fixture = CompletionFixture::new("filesystem-stage-completion-stage-link")?;
    let bytes = segment_bytes()?;
    let request = request(RecoveryStage::Segment, &bytes)?;
    let target = fixture.root().join("outside-stage");
    fs::write(&target, &bytes)?;
    symlink(&target, fixture.stage_path(RecoveryStage::Segment))?;
    let mut completer = fixture.completer()?;

    let error = execute_recovery_stage_completion(&mut completer, request)
        .err()
        .ok_or("symbolic recovery stage was followed")?;

    assert!(matches!(
        filesystem_stage_source(&error)?,
        FilesystemRecoveryStageError::Open {
            stage: RecoveryStage::Segment,
            ..
        }
    ));
    assert_eq!(fs::read(&target)?, bytes);
    assert!(fixture.stage_path(RecoveryStage::Segment).is_symlink());
    assert!(!fixture.pool_path(request).exists());
    drop(completer);
    fixture.remove()?;
    Ok(())
}

#[test]
fn symbolic_pool_is_never_followed_or_admitted() -> Result<(), Box<dyn Error>> {
    use std::os::unix::fs::symlink;

    let fixture = CompletionFixture::new("filesystem-stage-completion-pool-link")?;
    let bytes = catalog_bytes()?;
    let request = request(RecoveryStage::Catalog, &bytes)?;
    let target = fixture.root().join("outside-pool");
    fs::write(fixture.stage_path(RecoveryStage::Catalog), &bytes)?;
    fs::write(&target, &bytes)?;
    symlink(&target, fixture.pool_path(request))?;
    let mut completer = fixture.completer()?;

    let error = execute_recovery_stage_completion(&mut completer, request)
        .err()
        .ok_or("symbolic pool artifact was followed")?;

    assert!(matches!(
        filesystem_stage_source(&error)?,
        FilesystemRecoveryStageError::Open {
            stage: RecoveryStage::Catalog,
            ..
        }
    ));
    assert_eq!(fs::read(&target)?, bytes);
    assert!(fixture.pool_path(request).is_symlink());
    assert_eq!(fs::read(fixture.stage_path(RecoveryStage::Catalog))?, bytes);
    drop(completer);
    fixture.remove()?;
    Ok(())
}

#[test]
fn absent_stage_and_pool_refuse_without_creating_state() -> Result<(), Box<dyn Error>> {
    let fixture = CompletionFixture::new("filesystem-stage-completion-absent")?;
    let bytes = segment_bytes()?;
    let request = request(RecoveryStage::Segment, &bytes)?;
    let mut completer = fixture.completer()?;

    let error = execute_recovery_stage_completion(&mut completer, request)
        .err()
        .ok_or("absent stage and pool produced a receipt")?;

    assert!(matches!(
        error,
        RecoveryStageCompletionError::LinkOrAdmit {
            source: RecoveryStageCompletionStorageError::Missing {
                request: missing,
            },
            ..
        } if missing == request
    ));
    assert!(!fixture.stage_path(RecoveryStage::Segment).exists());
    assert!(!fixture.pool_path(request).exists());
    drop(completer);
    fixture.remove()?;
    Ok(())
}

#[test]
fn changed_stage_is_refused_before_a_pool_link_is_created() -> Result<(), Box<dyn Error>> {
    let fixture = CompletionFixture::new("filesystem-stage-completion-link-mismatch")?;
    let bytes = segment_bytes()?;
    let request = request(RecoveryStage::Segment, &bytes)?;
    fs::write(fixture.stage_path(RecoveryStage::Segment), b"different")?;
    let mut completer = fixture.completer()?;

    let error = completer
        .link_stage_or_admit_pool(request)
        .err()
        .ok_or("changed stage was linked into the pool")?;

    assert!(matches!(
        error,
        RecoveryStageCompletionStorageError::EvidenceMismatch {
            expected,
            observed,
        } if expected == request.evidence() && observed != expected
    ));
    assert!(!fixture.pool_path(request).exists());
    assert_eq!(
        fs::read(fixture.stage_path(RecoveryStage::Segment))?,
        b"different"
    );
    drop(completer);
    fixture.remove()?;
    Ok(())
}

fn filesystem_stage_source(
    error: &RecoveryStageCompletionError,
) -> Result<&FilesystemRecoveryStageError, &'static str> {
    let completion_source = match error {
        RecoveryStageCompletionError::SynchronizeStage { source, .. }
        | RecoveryStageCompletionError::VerifyPool { source, .. } => source,
        _ => return Err("filesystem stage refusal lost its completion phase"),
    };
    filesystem_storage_source(completion_source)
}

pub(super) fn filesystem_storage_source(
    error: &RecoveryStageCompletionStorageError,
) -> Result<&FilesystemRecoveryStageError, &'static str> {
    let RecoveryStageCompletionStorageError::Storage { source } = error else {
        return Err("filesystem stage refusal lost its storage boundary");
    };
    source
        .get_ref()
        .and_then(|source| source.downcast_ref::<FilesystemRecoveryStageError>())
        .ok_or("completion failure lost its typed filesystem stage source")
}
