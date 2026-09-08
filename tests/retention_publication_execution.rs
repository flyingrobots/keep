//! Ordered retention publication and consequential receipt laws.

#[path = "retention_publication_preparation/fixture.rs"]
pub mod fixture;
#[path = "retention_publication_storage/recording_storage.rs"]
pub mod recording_storage;
#[path = "retention_publication_execution/refusal_laws.rs"]
mod refusal_laws;
mod support;

use std::error::Error;

use keep::{
    AdmittedRetentionManifest, AdmittedRetentionRoot, RetentionGenerationExpectation,
    RetentionNamespaceAdmission, RetentionPublicationOutcome, RetentionPublicationPhase,
    RetentionPublicationPreparation, RootGeneration, execute_retention_publication,
    preflight_retention_transition, prepare_retention_publication,
};

use fixture::{manifest_bytes, root_bytes, with_snapshot};
use recording_storage::RecordingStorage;

#[test]
fn publication_executes_every_phase_and_returns_complete_coordinates() -> Result<(), Box<dyn Error>>
{
    let root_bytes = root_bytes()?;
    let candidate = AdmittedRetentionRoot::decode(&root_bytes)?;
    let namespace = candidate.root().namespace().digest();
    let root_generation = candidate.root().generation();
    let root_digest = candidate.digest();
    let profile = candidate.root().profile();
    let anchor_set_digest = candidate.anchor_set_digest();
    let preflight = with_snapshot(|snapshot| {
        preflight_retention_transition(
            RetentionGenerationExpectation::Absent,
            None,
            candidate,
            snapshot,
        )
    })??;
    let closure = preflight.closure();
    let preparation = prepare_retention_publication(preflight, None)?;
    let publication = preparation
        .publication()
        .ok_or("initial transition omitted publication artifacts")?;
    let liveness_generation = publication.liveness_generation();
    let manifest_digest = publication.manifest().digest();
    let mut storage = RecordingStorage::new();

    let receipt = execute_retention_publication(&mut storage, &preparation)?;

    assert_eq!(storage.verification_count(), 1);
    assert_eq!(storage.observed(), RetentionPublicationPhase::ALL);
    assert_eq!(receipt.outcome(), RetentionPublicationOutcome::Published);
    assert_eq!(
        receipt.namespace_admission(),
        Some(RetentionNamespaceAdmission::Created)
    );
    assert_eq!(receipt.namespace(), namespace);
    assert_eq!(receipt.expected(), RetentionGenerationExpectation::Absent);
    assert_eq!(receipt.observed(), None);
    assert_eq!(receipt.root_generation(), root_generation);
    assert_eq!(receipt.root_digest(), root_digest);
    assert_eq!(receipt.liveness_generation(), liveness_generation);
    assert_eq!(receipt.manifest_digest(), manifest_digest);
    assert_eq!(receipt.profile(), profile);
    assert_eq!(receipt.anchor_set_digest(), anchor_set_digest);
    assert_eq!(receipt.closure_digest(), closure.digest());
    assert_eq!(receipt.catalog_generation(), closure.catalog_generation());
    assert_eq!(receipt.catalog_digest(), closure.catalog_digest());
    Ok(())
}

#[test]
fn exact_retry_revalidates_authority_without_publication_mutation() -> Result<(), Box<dyn Error>> {
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
    let mut storage = RecordingStorage::already_committed();

    let receipt = execute_retention_publication(&mut storage, &preparation)?;

    assert_eq!(storage.verification_count(), 1);
    assert!(storage.observed().is_empty());
    assert_eq!(
        receipt.outcome(),
        RetentionPublicationOutcome::AlreadyCommitted
    );
    assert_eq!(receipt.namespace_admission(), None);
    assert_eq!(receipt.observed(), Some(RootGeneration::INITIAL));
    assert_eq!(
        receipt.liveness_generation(),
        current_manifest.manifest().generation()
    );
    assert_eq!(receipt.manifest_digest(), current_manifest.digest());
    Ok(())
}

#[test]
fn existing_namespace_skips_only_the_parent_directory_synchronization() -> Result<(), Box<dyn Error>>
{
    let root_bytes = root_bytes()?;
    let preparation = initial_preparation(&root_bytes)?;
    let mut storage = RecordingStorage::existing_namespace();
    let expected = RetentionPublicationPhase::ALL
        .into_iter()
        .filter(|phase| *phase != RetentionPublicationPhase::SynchronizeRootsAfterNamespace)
        .collect::<Vec<_>>();

    let receipt = execute_retention_publication(&mut storage, &preparation)?;

    assert_eq!(storage.observed(), expected);
    assert_eq!(
        receipt.namespace_admission(),
        Some(RetentionNamespaceAdmission::Existing)
    );
    Ok(())
}

pub(crate) fn initial_preparation(
    root_bytes: &[u8],
) -> Result<RetentionPublicationPreparation<'_>, Box<dyn Error>> {
    let candidate = AdmittedRetentionRoot::decode(root_bytes)?;
    let preflight = with_snapshot(|snapshot| {
        preflight_retention_transition(
            RetentionGenerationExpectation::Absent,
            None,
            candidate,
            snapshot,
        )
    })??;
    prepare_retention_publication(preflight, None).map_err(Into::into)
}
