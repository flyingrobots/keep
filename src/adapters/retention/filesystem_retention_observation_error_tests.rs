//! This module owns publication's recovery-observation error contract.

use std::error::Error;
use std::fs;
use std::io;

use super::filesystem_retention_test_fixture::{
    ROOT_HEX, fixture, initial_preparation, open_authority,
};
use super::{RetentionCurrentStateRefusal as Refusal, RetentionPublicationError};
use crate::execute_retention_publication;

fn observation(error: &RetentionPublicationError) -> Result<&io::Error, Box<dyn Error>> {
    let RetentionPublicationError::CurrentVerification { source } = error else {
        return Err(format!("observation must refuse current verification: {error:?}").into());
    };
    let Some(refusal @ Refusal::RecoveryObservationRefused { .. }) = source
        .get_ref()
        .and_then(|source| source.downcast_ref::<Refusal>())
    else {
        return Err(format!("publication must identify recovery observation: {error:?}").into());
    };
    refusal
        .source()
        .and_then(|source| source.downcast_ref::<io::Error>())
        .ok_or_else(|| {
            "observation must expose its original I/O source through Error::source".into()
        })
}

// Size: medium. Oracle: publication identifies recovery observation and preserves its typed cause.
// Delete when this publication boundary is removed or stronger error-chain coverage subsumes it.
#[test]
fn publication_preserves_the_recovery_observation_refusal() -> Result<(), Box<dyn Error>> {
    let (sandbox, mut authority) = open_authority("retention-observation-typed-source")?;
    fs::write(sandbox.path().join("retention/unknown"), b"evidence")?;
    let root = fixture(ROOT_HEX)?;
    let error = execute_retention_publication(&mut authority, &initial_preparation(&root)?)
        .err()
        .ok_or("unknown evidence was admitted")?;
    let source = observation(&error)?;
    assert!(
        matches!(
            source
                .get_ref()
                .and_then(|source| source.downcast_ref::<Refusal>()),
            Some(Refusal::UnknownRetentionEntry)
        ),
        "original namespace refusal must survive: {error:?}"
    );
    Ok(())
}

// Size: medium. Oracle: a no-follow stage-open failure preserves its exact OS cause through observation.
// Delete when stage observation is removed or stronger kernel-fault coverage subsumes it.
#[cfg(unix)]
#[test]
fn publication_preserves_the_recovery_observation_io_cause() -> Result<(), Box<dyn Error>> {
    let (sandbox, mut authority) = open_authority("retention-observation-io-source")?;
    std::os::unix::fs::symlink("missing-stage", sandbox.path().join("retention/root.next"))?;
    let root = fixture(ROOT_HEX)?;
    let error = execute_retention_publication(&mut authority, &initial_preparation(&root)?)
        .err()
        .ok_or("symlink stage was admitted")?;
    let source = observation(&error)?;
    assert_eq!(
        source.raw_os_error(),
        Some(rustix::io::Errno::LOOP.raw_os_error()),
        "original no-follow error must survive: {error:?}"
    );
    Ok(())
}
