//! Retention publication execution refusal laws.

use std::error::Error;
use std::io;

use keep::{
    AdmittedRetentionManifest, AdmittedRetentionRoot, RetentionGenerationExpectation,
    RetentionPublicationError, RetentionPublicationPhase, RetentionTransitionDisposition,
    execute_retention_publication, preflight_retention_transition, prepare_retention_publication,
};

use crate::fixture::{manifest_bytes, root_bytes, with_snapshot};
use crate::recording_storage::RecordingStorage;
use crate::support::require_error;

#[test]
fn current_authority_refusal_precedes_every_publication_phase() -> Result<(), Box<dyn Error>> {
    let root_bytes = root_bytes()?;
    let preparation = crate::initial_preparation(&root_bytes)?;
    let mut storage = RecordingStorage::verification_failure();

    let error = require_error(
        execute_retention_publication(&mut storage, &preparation),
        "authority refusal returned a receipt",
    )?;

    assert!(matches!(
        error,
        RetentionPublicationError::CurrentVerification { source }
            if source.kind() == io::ErrorKind::PermissionDenied
    ));
    assert_eq!(storage.verification_count(), 1);
    assert!(storage.observed().is_empty());
    Ok(())
}

#[test]
fn every_phase_failure_stops_before_all_later_mutation() -> Result<(), Box<dyn Error>> {
    let mut expected = Vec::new();
    for failing_phase in RetentionPublicationPhase::ALL {
        expected.push(failing_phase);
        let root_bytes = root_bytes()?;
        let preparation = crate::initial_preparation(&root_bytes)?;
        let mut storage = RecordingStorage::failing_at(failing_phase);

        let error = require_error(
            execute_retention_publication(&mut storage, &preparation),
            "phase failure returned a receipt",
        )?;

        assert!(matches!(
            error,
            RetentionPublicationError::Storage { phase, source }
                if phase == failing_phase && source.kind() == io::ErrorKind::Other
        ));
        assert_eq!(storage.verification_count(), 1);
        assert_eq!(storage.observed(), expected);
    }
    Ok(())
}

#[test]
fn changed_disposition_refuses_before_publication_mutation() -> Result<(), Box<dyn Error>> {
    let root_bytes = root_bytes()?;
    let current = AdmittedRetentionRoot::decode(&root_bytes)?;
    let candidate = AdmittedRetentionRoot::decode(&root_bytes)?;
    let manifest_bytes = manifest_bytes()?;
    let current_manifest = AdmittedRetentionManifest::decode(&manifest_bytes)?;
    let preflight = with_snapshot(|snapshot| {
        preflight_retention_transition(
            RetentionGenerationExpectation::Absent,
            Some(&current),
            candidate,
            snapshot,
        )
    })??;
    let preparation = prepare_retention_publication(preflight, Some(&current_manifest))?;
    let mut storage = RecordingStorage::new();

    let error = require_error(
        execute_retention_publication(&mut storage, &preparation),
        "changed disposition returned a receipt",
    )?;

    assert!(matches!(
        error,
        RetentionPublicationError::DispositionMismatch {
            prepared: RetentionTransitionDisposition::AlreadyCommitted,
            observed: RetentionTransitionDisposition::Publish,
        }
    ));
    assert!(storage.observed().is_empty());
    Ok(())
}
