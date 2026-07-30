//! Retention publication preparation refusal laws.

use std::error::Error;

use keep::{
    AdmittedRetentionManifest, AdmittedRetentionRoot, LivenessGeneration, LivenessGenerationError,
    RetentionGenerationExpectation, RetentionManifest, RetentionPublicationPreparationError,
    preflight_retention_transition, prepare_retention_publication,
};

use super::fixture::{initial_root, manifest_bytes, root_bytes, with_snapshot};
use crate::support::require_error;

#[test]
fn manifest_disagreement_refuses_before_global_artifact_construction() -> Result<(), Box<dyn Error>>
{
    let root_bytes = root_bytes()?;
    let candidate = AdmittedRetentionRoot::decode(&root_bytes)?;
    let manifest_bytes = manifest_bytes()?;
    let current_manifest = AdmittedRetentionManifest::decode(&manifest_bytes)?;
    let preflight = with_snapshot(|snapshot| {
        preflight_retention_transition(
            RetentionGenerationExpectation::Absent,
            None,
            candidate,
            snapshot,
        )
    })??;

    let error = require_error(
        prepare_retention_publication(preflight, Some(&current_manifest)),
        "manifest disagreement prepared a publication",
    )?;

    assert!(matches!(
        error,
        RetentionPublicationPreparationError::ManifestSuccessorMismatch { .. }
    ));
    Ok(())
}

#[test]
fn exhausted_liveness_generation_refuses_before_entry_replacement() -> Result<(), Box<dyn Error>> {
    let root_bytes = root_bytes()?;
    let template = AdmittedRetentionRoot::decode(&root_bytes)?;
    let manifest_bytes = manifest_bytes()?;
    let current_manifest = AdmittedRetentionManifest::decode(&manifest_bytes)?;
    let maximum_manifest = RetentionManifest::new(
        LivenessGeneration::new(u64::MAX)?,
        Some(current_manifest.digest()),
        current_manifest.manifest().entries().to_vec(),
    )?;
    let maximum_bytes = keep::CanonicalRetentionManifest::from_manifest(&maximum_manifest)?;
    let maximum = AdmittedRetentionManifest::decode(maximum_bytes.encoded())?;
    let candidate_bytes = initial_root(b"exhaustion-candidate", &template)?;
    let candidate = AdmittedRetentionRoot::decode(candidate_bytes.encoded())?;
    let preflight = with_snapshot(|snapshot| {
        preflight_retention_transition(
            RetentionGenerationExpectation::Absent,
            None,
            candidate,
            snapshot,
        )
    })??;

    let error = require_error(
        prepare_retention_publication(preflight, Some(&maximum)),
        "exhausted liveness generation prepared a publication",
    )?;

    assert!(matches!(
        error,
        RetentionPublicationPreparationError::LivenessGeneration {
            source: LivenessGenerationError::Exhausted { current: u64::MAX }
        }
    ));
    Ok(())
}
