//! Successor retention publication preparation laws.

use std::error::Error;

use keep::{
    AdmittedRetentionManifest, AdmittedRetentionRoot, ChecksummedRetentionHead,
    RetentionGenerationExpectation, RetentionTransitionDisposition, RootGeneration,
    preflight_retention_transition, prepare_retention_publication,
};

use super::fixture::{initial_root, manifest_bytes, root_bytes, successor_root, with_snapshot};

#[test]
fn successor_replaces_only_the_selected_manifest_entry() -> Result<(), Box<dyn Error>> {
    let root_bytes = root_bytes()?;
    let current = AdmittedRetentionRoot::decode(&root_bytes)?;
    let candidate_bytes = successor_root(&current)?;
    let candidate = AdmittedRetentionRoot::decode(candidate_bytes.encoded())?;
    let manifest_bytes = manifest_bytes()?;
    let current_manifest = AdmittedRetentionManifest::decode(&manifest_bytes)?;
    let candidate_digest = candidate.digest();
    let preflight = with_snapshot(|snapshot| {
        preflight_retention_transition(
            RetentionGenerationExpectation::Current(RootGeneration::INITIAL),
            Some(&current),
            candidate,
            snapshot,
        )
    })??;

    let preparation = prepare_retention_publication(preflight, Some(&current_manifest))?;
    assert_eq!(
        preparation.expected(),
        RetentionGenerationExpectation::Current(RootGeneration::INITIAL)
    );
    assert_eq!(preparation.observed(), Some(RootGeneration::INITIAL));
    assert_eq!(
        preparation.disposition(),
        RetentionTransitionDisposition::Publish
    );
    let publication = preparation
        .publication()
        .ok_or("successor transition did not prepare publication")?;
    let manifest = AdmittedRetentionManifest::decode(publication.manifest().encoded())?;
    let head = ChecksummedRetentionHead::decode(publication.head().encoded())?;
    let entry = manifest
        .manifest()
        .entries()
        .first()
        .ok_or("successor manifest omitted the namespace")?;

    assert_eq!(manifest.manifest().generation().get(), 2);
    assert_eq!(
        manifest.manifest().predecessor(),
        Some(current_manifest.digest())
    );
    assert_eq!(entry.root_generation().get(), 2);
    assert_eq!(entry.root_digest(), candidate_digest);
    assert_eq!(head.head().manifest_digest(), manifest.digest());
    Ok(())
}

#[test]
fn new_namespace_is_inserted_without_changing_existing_entry() -> Result<(), Box<dyn Error>> {
    let root_bytes = root_bytes()?;
    let template = AdmittedRetentionRoot::decode(&root_bytes)?;
    let candidate_bytes = initial_root(b"second-namespace", &template)?;
    let candidate = AdmittedRetentionRoot::decode(candidate_bytes.encoded())?;
    let candidate_namespace = candidate.root().namespace().digest();
    let manifest_bytes = manifest_bytes()?;
    let current_manifest = AdmittedRetentionManifest::decode(&manifest_bytes)?;
    let existing = *current_manifest
        .manifest()
        .entries()
        .first()
        .ok_or("fixture manifest omitted its root")?;
    let preflight = with_snapshot(|snapshot| {
        preflight_retention_transition(
            RetentionGenerationExpectation::Absent,
            None,
            candidate,
            snapshot,
        )
    })??;

    let preparation = prepare_retention_publication(preflight, Some(&current_manifest))?;
    assert_eq!(
        preparation.expected(),
        RetentionGenerationExpectation::Absent
    );
    assert_eq!(preparation.observed(), None);
    assert_eq!(
        preparation.disposition(),
        RetentionTransitionDisposition::Publish
    );
    let publication = preparation
        .publication()
        .ok_or("new namespace did not prepare publication")?;
    let manifest = AdmittedRetentionManifest::decode(publication.manifest().encoded())?;

    assert_eq!(manifest.manifest().entry_count(), 2);
    assert!(manifest.manifest().entries().contains(&existing));
    assert!(
        manifest
            .manifest()
            .entries()
            .iter()
            .any(|entry| entry.namespace() == candidate_namespace)
    );
    Ok(())
}

#[test]
fn exact_retry_prepares_no_new_global_artifacts() -> Result<(), Box<dyn Error>> {
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

    assert_eq!(
        preparation.expected(),
        RetentionGenerationExpectation::Absent
    );
    assert_eq!(preparation.observed(), Some(RootGeneration::INITIAL));
    assert_eq!(
        preparation.disposition(),
        RetentionTransitionDisposition::AlreadyCommitted
    );
    assert!(preparation.publication().is_none());
    assert_eq!(preparation.candidate().digest(), current.digest());
    assert_eq!(preparation.closure().usage().node_count(), 2);
    Ok(())
}
