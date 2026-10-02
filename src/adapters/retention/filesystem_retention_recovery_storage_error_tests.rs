//! This module owns typed recovery refusals after retained-stage admission.

use std::error::Error;
use std::fs;
use std::io;

use super::{RetentionRecoveryContext, RetentionRecoveryObservation};
use crate::adapters::retention::filesystem_retention_test_fixture::{
    ROOT_HEX, drive_publication, fixture, initial_preparation, open_authority, retention_witness,
};
use crate::adapters::retention::{
    RetentionRecordRefusal, RetentionRecoveryError, RetentionRecoveryStep,
    execute_retention_recovery, plan_retention_recovery,
};

#[derive(Clone, Copy)]
enum Damage {
    Bytes,
    Identity,
    Missing,
}

// Size: medium. Oracle: byte disagreement remains distinguishable from inode substitution.
// Delete when recovery storage is removed or stronger public error laws subsume this scenario.
#[test]
fn changed_stage_bytes_preserve_the_recovery_record_refusal() -> Result<(), Box<dyn Error>> {
    let error = refused("recovery-error-bytes", Damage::Bytes)?;
    assert_eq!(
        cause::<RetentionRecordRefusal>(&error),
        Some(&RetentionRecordRefusal::Bytes),
        "recovery must preserve the exact byte refusal: {error:?}"
    );
    Ok(())
}

// Size: medium. Oracle: byte-identical inode substitution remains a typed identity refusal.
// Delete when recovery storage is removed or stronger public error laws subsume this scenario.
#[test]
fn substituted_stage_preserves_the_recovery_record_refusal() -> Result<(), Box<dyn Error>> {
    let error = refused("recovery-error-identity", Damage::Identity)?;
    assert_eq!(
        cause::<RetentionRecordRefusal>(&error),
        Some(&RetentionRecordRefusal::KindLengthOrIdentity),
        "recovery must preserve the exact identity refusal: {error:?}"
    );
    Ok(())
}

// Size: medium. Oracle: missing retained stages preserve the original filesystem error.
// Delete when recovery storage is removed or stronger public error laws subsume this scenario.
#[test]
fn a_missing_stage_preserves_the_recovery_io_cause() -> Result<(), Box<dyn Error>> {
    let error = refused("recovery-error-missing", Damage::Missing)?;
    assert_eq!(
        cause::<io::Error>(&error).map(io::Error::kind),
        Some(io::ErrorKind::NotFound),
        "recovery must preserve the missing-stage I/O error: {error:?}"
    );
    assert_eq!(
        cause::<io::Error>(&error).and_then(io::Error::raw_os_error),
        Some(rustix::io::Errno::NOENT.raw_os_error()),
        "recovery must preserve the original OS error code: {error:?}"
    );
    Ok(())
}

fn refused(name: &str, damage: Damage) -> Result<RetentionRecoveryError, Box<dyn Error>> {
    let (sandbox, mut authority) = open_authority(name)?;
    let bytes = fixture(ROOT_HEX)?;
    let preparation = initial_preparation(&bytes)?;
    drive_publication(&mut authority, &preparation, 2)?;
    let observation = RetentionRecoveryObservation::observe(
        &authority.retention,
        &authority.roots,
        &authority.manifests,
    )?;
    let plan = plan_retention_recovery(observation.evidence())?;
    authority.recovery = Some(RetentionRecoveryContext::reopen(
        &authority.retention,
        &observation,
    )?);
    let path = sandbox.path().join("retention/root.next");
    match damage {
        Damage::Bytes => {
            let mut changed = fs::read(&path)?;
            *changed.last_mut().ok_or("empty root stage")? ^= 1;
            fs::write(&path, changed)?;
        }
        Damage::Identity => {
            let replacement = path.with_extension("replacement");
            fs::copy(&path, &replacement)?;
            fs::rename(replacement, &path)?;
        }
        Damage::Missing => fs::remove_file(&path)?,
    }
    let before = retention_witness(sandbox.path())?;

    let error = execute_retention_recovery(&mut authority, &plan)
        .err()
        .ok_or("damaged recovery stage accepted")?;

    assert_eq!(
        error.step(),
        RetentionRecoveryStep::LinkRoot,
        "the first root transition must refuse"
    );
    assert_eq!(
        retention_witness(sandbox.path())?,
        before,
        "recovery refusal must preserve all retained evidence"
    );
    Ok(error)
}

fn cause<'a, T: Error + 'static>(mut error: &'a (dyn Error + 'static)) -> Option<&'a T> {
    loop {
        if let Some(cause) = error.downcast_ref::<T>() {
            return Some(cause);
        }
        error = error.source()?;
    }
}

// Size: medium. Oracle: cleanup may remove only the exact retained source with proven pool evidence.
// Bug regression; delete only when a stronger runtime identity law subsumes this boundary.
#[test]
fn substituted_cleanup_source_refuses_before_unlink() -> Result<(), Box<dyn Error>> {
    let (sandbox, mut authority) = open_authority("cleanup-source-substitution")?;
    let root = fixture(ROOT_HEX)?;
    let preparation = initial_preparation(&root)?;
    drive_publication(&mut authority, &preparation, 15)?;
    let observation = RetentionRecoveryObservation::observe(
        &authority.retention,
        &authority.roots,
        &authority.manifests,
    )?;
    let plan = plan_retention_recovery(observation.evidence())?;
    authority.recovery = Some(RetentionRecoveryContext::reopen(
        &authority.retention,
        &observation,
    )?);
    let stage = sandbox.path().join("retention/root.next");
    let replacement = stage.with_extension("replacement");
    fs::copy(&stage, &replacement)?;
    fs::rename(replacement, stage)?;
    let before = retention_witness(sandbox.path())?;
    let error = execute_retention_recovery(&mut authority, &plan)
        .err()
        .ok_or("cleanup deleted substituted source")?;
    assert_eq!(
        error.step(),
        RetentionRecoveryStep::RemoveRootStage,
        "source identity must be guarded before cleanup"
    );
    assert_eq!(
        cause::<RetentionRecordRefusal>(&error),
        Some(&RetentionRecordRefusal::KindLengthOrIdentity),
        "source substitution must retain its typed cause"
    );
    assert!(
        error.executed().is_empty(),
        "no preceding recovery step completed"
    );
    let progress = error
        .progress()
        .ok_or("source refusal omitted pre-effect status")?;
    assert_eq!(
        progress.boundary(),
        crate::RetentionStorageBoundary::SourceVerification
    );
    assert!(
        progress.known_effects().is_empty(),
        "source guard precedes namespace effects"
    );
    assert_eq!(progress.uncertain_effect(), None);
    assert_eq!(
        retention_witness(sandbox.path())?,
        before,
        "pre-effect source refusal must preserve source and pool evidence"
    );
    Ok(())
}

// Size: medium. Oracle: failed directory synchronization cannot roll back an earlier unlink.
// Real filesystem unlink, deterministic injected EIO before directory sync; not a power-loss test.
// Delete only when stronger public recovery failure/restart laws subsume this exact effect boundary.
#[test]
fn cleanup_sync_failure_reports_removed_stage_and_preserved_pool() -> Result<(), Box<dyn Error>> {
    use crate::adapters::retention::{
        FilesystemRetentionRecoveryError, RetentionEffectDurability as Durability,
        RetentionNamespaceEffect as Effect, RetentionStorageBoundary as Boundary,
    };
    let (sandbox, mut authority) = open_authority("cleanup-sync-failure")?;
    let root = fixture(ROOT_HEX)?;
    let preparation = initial_preparation(&root)?;
    drive_publication(&mut authority, &preparation, 15)?;
    authority.recovery_sync_failure = Some(Boundary::RetentionSynchronization);
    let error = match authority.recover() {
        Err(FilesystemRetentionRecoveryError::Execute { source }) => source,
        result => return Err(format!("expected cleanup sync failure: {result:?}").into()),
    };
    assert_eq!(error.step(), RetentionRecoveryStep::RemoveRootStage);
    assert!(
        error.executed().is_empty(),
        "no preceding step completed despite the failing capability's unlink"
    );
    let progress = error
        .progress()
        .ok_or("missing failing-capability effects after unlink")?;
    assert_eq!(progress.boundary(), Boundary::RetentionSynchronization);
    assert_eq!(
        progress
            .known_effects()
            .iter()
            .map(|effect| (effect.effect(), effect.durability()))
            .collect::<Vec<_>>(),
        [(Effect::StageRemoved, Durability::Unconfirmed)]
    );
    assert_eq!(progress.uncertain_effect(), None);
    assert_eq!(
        cause::<io::Error>(&error).and_then(io::Error::raw_os_error),
        Some(rustix::io::Errno::IO.raw_os_error())
    );
    assert!(
        !sandbox.path().join("retention/root.next").exists(),
        "unlink already occurred"
    );
    let pool = crate::adapters::retention::filesystem_retention_test_fixture::root_pool_path(
        sandbox.path(),
        preparation.candidate(),
    );
    assert_eq!(
        fs::read(pool)?,
        root,
        "exact pool evidence survives cleanup failure"
    );
    assert!(
        sandbox.path().join("retention/manifest.next").exists(),
        "later cleanup must not execute"
    );
    drop(authority);
    let admission =
        crate::adapters::FilesystemVersionTwoAdmission::reopen_unchecked_for_repository_tasks(
            sandbox.path(),
        )?;
    let mut restarted =
        crate::adapters::retention::FilesystemRetentionPublicationAuthority::open(admission)?;
    let retry = restarted.recover()?;
    assert_eq!(
        retry.executed(),
        [RetentionRecoveryStep::RemoveManifestStage],
        "retry must freshly observe the completed unlink"
    );
    Ok(())
}

// Size: medium. Oracle: an observed stage identity remains binding when execution reopens it.
// Bug regression; delete only when stronger recovery observation-to-execution coverage subsumes it.
#[test]
fn substitution_before_reopening_refuses_without_rebinding_evidence() -> Result<(), Box<dyn Error>>
{
    let (sandbox, mut authority) = open_authority("recovery-observation-substitution")?;
    let root = fixture(ROOT_HEX)?;
    let preparation = initial_preparation(&root)?;
    drive_publication(&mut authority, &preparation, 2)?;
    let observation = RetentionRecoveryObservation::observe(
        &authority.retention,
        &authority.roots,
        &authority.manifests,
    )?;
    let stage = sandbox.path().join("retention/root.next");
    let replacement = stage.with_extension("replacement");
    fs::copy(&stage, &replacement)?;
    fs::rename(replacement, stage)?;
    let before = retention_witness(sandbox.path())?;
    let error = RetentionRecoveryContext::reopen(&authority.retention, &observation)
        .err()
        .ok_or("reopening silently rebound observed stage identity")?;
    assert_eq!(
        cause::<RetentionRecordRefusal>(&error),
        Some(&RetentionRecordRefusal::KindLengthOrIdentity),
        "observed substitution must preserve its identity diagnostic"
    );
    assert_eq!(retention_witness(sandbox.path())?, before);
    Ok(())
}
