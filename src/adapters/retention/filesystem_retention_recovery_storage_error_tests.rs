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
