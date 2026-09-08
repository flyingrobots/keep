//! Filesystem retention successor-generation and superseded-retry laws.

use std::error::Error;
use std::fs;
use std::io;

use super::filesystem_retention_test_fixture::{
    ROOT_HEX, fixture, head_path, initial_generation, initial_preparation, manifest_pool_path,
    open_authority, refusal, retention_witness, root_pool_path, successor_preparation,
    successor_root,
};
use super::{
    AdmittedRetentionManifest, AdmittedRetentionRoot, ChecksummedRetentionHead,
    RetentionCurrentStateRefusal, RetentionNamespaceAdmission, RetentionPublicationError,
    RetentionPublicationOutcome, RetentionPublicationStorage,
};
use crate::execute_retention_publication;

#[test]
fn successor_publication_over_existing_head_publishes_exact_successor() -> Result<(), Box<dyn Error>>
{
    let (sandbox, mut authority) = open_authority("filesystem-retention-successor")?;
    let root_bytes = fixture(ROOT_HEX)?;
    let initial = initial_preparation(&root_bytes)?;
    let _published = execute_retention_publication(&mut authority, &initial)?;
    let generation_one = retention_witness(sandbox.path())?;
    let current = authority
        .observe_current()?
        .ok_or("published retention head was not observed")?;
    let current_root = AdmittedRetentionRoot::decode(&root_bytes)?;
    let current_manifest = AdmittedRetentionManifest::decode(current.manifest_bytes())?;
    let candidate = successor_root(&current_root)?;
    let preparation = successor_preparation(&current_root, &current_manifest, candidate.encoded())?;

    let receipt = execute_retention_publication(&mut authority, &preparation)?;

    assert_eq!(receipt.outcome(), RetentionPublicationOutcome::Published);
    assert_eq!(
        receipt.namespace_admission(),
        Some(RetentionNamespaceAdmission::Existing)
    );
    let head_bytes = fs::read(head_path(sandbox.path()))?;
    let head = ChecksummedRetentionHead::decode(&head_bytes)?;
    assert_eq!(
        head.head().generation(),
        current_manifest.manifest().generation().successor()?
    );
    assert_eq!(head.head().predecessor(), Some(current_manifest.digest()));
    assert_eq!(head.head().manifest_digest(), preparation.manifest_digest());
    assert_eq!(
        fs::read(root_pool_path(sandbox.path(), preparation.candidate()))?,
        candidate.encoded()
    );
    assert!(manifest_pool_path(sandbox.path(), &preparation).exists());
    let generation_two = retention_witness(sandbox.path())?;
    for entry in &generation_one {
        if !entry.0.to_string_lossy().ends_with("HEAD") {
            assert!(
                generation_two.contains(entry),
                "generation-one evidence changed"
            );
        }
    }
    Ok(())
}

#[test]
fn superseded_candidate_refuses_once_a_successor_is_current() -> Result<(), Box<dyn Error>> {
    let (sandbox, mut authority) = open_authority("filesystem-retention-superseded")?;
    let root_bytes = fixture(ROOT_HEX)?;
    let _published =
        execute_retention_publication(&mut authority, &initial_preparation(&root_bytes)?)?;
    let current = authority
        .observe_current()?
        .ok_or("published retention head was not observed")?;
    let current_root = AdmittedRetentionRoot::decode(&root_bytes)?;
    let current_manifest = AdmittedRetentionManifest::decode(current.manifest_bytes())?;
    let candidate = successor_root(&current_root)?;
    let successor = successor_preparation(&current_root, &current_manifest, candidate.encoded())?;
    let _advanced = execute_retention_publication(&mut authority, &successor)?;
    let after_successor = retention_witness(sandbox.path())?;
    let stale_retry = initial_preparation(&root_bytes)?;

    let error = execute_retention_publication(&mut authority, &stale_retry)
        .err()
        .ok_or("superseded generation-one candidate was unexpectedly accepted")?;

    let RetentionPublicationError::CurrentVerification { source } = error else {
        return Err("superseded candidate refused outside current-state verification".into());
    };
    assert_eq!(source.kind(), io::ErrorKind::InvalidData);
    assert!(matches!(
        refusal(&source),
        Some(RetentionCurrentStateRefusal::Superseded { .. })
    ));
    assert_eq!(retention_witness(sandbox.path())?, after_successor);
    Ok(())
}

#[test]
fn expected_current_generation_refuses_when_no_head_is_published() -> Result<(), Box<dyn Error>> {
    let (sandbox, mut authority) = open_authority("filesystem-retention-absent-predecessor")?;
    let root_bytes = fixture(ROOT_HEX)?;
    let current_root = AdmittedRetentionRoot::decode(&root_bytes)?;
    let manifest_bytes = fixture(super::filesystem_retention_test_fixture::MANIFEST_HEX)?;
    let current_manifest = AdmittedRetentionManifest::decode(&manifest_bytes)?;
    let candidate = successor_root(&current_root)?;
    let preparation = successor_preparation(&current_root, &current_manifest, candidate.encoded())?;
    assert_eq!(preparation.observed(), Some(initial_generation()));

    let error = RetentionPublicationStorage::verify_current(&mut authority, &preparation)
        .err()
        .ok_or("successor over an absent head was unexpectedly admitted")?;

    assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    assert!(matches!(
        refusal(&error),
        Some(RetentionCurrentStateRefusal::ExpectedCurrentOverAbsentHead)
    ));
    assert!(authority.observe_current()?.is_none());
    assert!(!head_path(sandbox.path()).exists());
    Ok(())
}
