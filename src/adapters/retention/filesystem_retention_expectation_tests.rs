//! Filesystem retention expectation laws: the head and namespace must match the claim.

use std::error::Error;
use std::fs;
use std::io;

use super::filesystem_retention_test_fixture::{
    ROOT_HEX, fixture, head_path, initial_preparation, initial_root, new_namespace_preparation,
    open_authority, refusal, retention_witness, root_pool_path, successor_preparation,
    successor_root,
};
use super::{
    AdmittedRetentionManifest, AdmittedRetentionRoot, RetentionCurrentStateRefusal,
    RetentionPublicationStorage,
};
use crate::{RetentionNamespace, RetentionRoot, execute_retention_publication};

#[test]
fn absent_head_with_retention_artifacts_refuses_as_recovery() -> Result<(), Box<dyn Error>> {
    let (sandbox, mut authority) = open_authority("filesystem-retention-absent-head-artifacts")?;
    let root_bytes = fixture(ROOT_HEX)?;
    let published = initial_preparation(&root_bytes)?;
    let _receipt = execute_retention_publication(&mut authority, &published)?;
    fs::remove_file(head_path(sandbox.path()))?;
    let before = retention_witness(sandbox.path())?;
    let retry = initial_preparation(&root_bytes)?;

    let error = RetentionPublicationStorage::verify_current(&mut authority, &retry)
        .err()
        .ok_or("absent head over populated pools was unexpectedly treated as empty")?;

    assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    assert!(matches!(
        refusal(&error),
        Some(RetentionCurrentStateRefusal::HeadAbsentWithArtifacts)
    ));
    assert_eq!(retention_witness(sandbox.path())?, before);
    Ok(())
}

#[test]
fn absent_expectation_refuses_an_orphan_directory_for_a_new_namespace() -> Result<(), Box<dyn Error>>
{
    let (sandbox, mut authority) = open_authority("filesystem-retention-orphan-new-namespace")?;
    let root_bytes = fixture(ROOT_HEX)?;
    let _published =
        execute_retention_publication(&mut authority, &initial_preparation(&root_bytes)?)?;
    let current = authority
        .observe_current()?
        .ok_or("published retention head was not observed")?;
    let current_manifest = AdmittedRetentionManifest::decode(current.manifest_bytes())?;
    let template = AdmittedRetentionRoot::decode(&root_bytes)?;
    let candidate = initial_root(b"second-namespace", &template)?;
    let preparation = new_namespace_preparation(&current_manifest, candidate.encoded())?;
    let namespace = root_pool_path(sandbox.path(), preparation.candidate())
        .parent()
        .ok_or("root pool path has no namespace directory")?
        .to_path_buf();
    fs::create_dir(&namespace)?;
    let before = retention_witness(sandbox.path())?;

    let error = RetentionPublicationStorage::verify_current(&mut authority, &preparation)
        .err()
        .ok_or("orphan namespace directory was unexpectedly admitted for an Absent expectation")?;

    assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    assert!(matches!(
        super::filesystem_retention_test_fixture::refusal(&error),
        Some(super::RetentionCurrentStateRefusal::NamespaceExpectationViolated)
    ));
    assert_eq!(retention_witness(sandbox.path())?, before);
    Ok(())
}

#[test]
fn current_expectation_refuses_when_the_namespace_directory_is_absent() -> Result<(), Box<dyn Error>>
{
    let (sandbox, mut authority) = open_authority("filesystem-retention-current-namespace-absent")?;
    let root_bytes = fixture(ROOT_HEX)?;
    let _published =
        execute_retention_publication(&mut authority, &initial_preparation(&root_bytes)?)?;
    let current = authority
        .observe_current()?
        .ok_or("published retention head was not observed")?;
    let current_root = AdmittedRetentionRoot::decode(&root_bytes)?;
    let current_manifest = AdmittedRetentionManifest::decode(current.manifest_bytes())?;
    let namespace = root_pool_path(sandbox.path(), &current_root)
        .parent()
        .ok_or("root pool path has no namespace directory")?
        .to_path_buf();
    fs::remove_dir_all(&namespace)?;
    let candidate = successor_root(&current_root)?;
    let preparation = successor_preparation(&current_root, &current_manifest, candidate.encoded())?;

    let error = RetentionPublicationStorage::verify_current(&mut authority, &preparation)
        .err()
        .ok_or("successor over an absent namespace directory was unexpectedly admitted")?;

    assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    assert!(matches!(
        super::filesystem_retention_test_fixture::refusal(&error),
        Some(super::RetentionCurrentStateRefusal::NamespaceExpectationViolated)
    ));
    Ok(())
}

#[test]
fn successor_refuses_when_the_predecessor_root_file_is_absent() -> Result<(), Box<dyn Error>> {
    let (sandbox, mut authority) = open_authority("filesystem-retention-predecessor-absent")?;
    let root_bytes = fixture(ROOT_HEX)?;
    let _published =
        execute_retention_publication(&mut authority, &initial_preparation(&root_bytes)?)?;
    let current = authority
        .observe_current()?
        .ok_or("published retention head was not observed")?;
    let current_root = AdmittedRetentionRoot::decode(&root_bytes)?;
    let current_manifest = AdmittedRetentionManifest::decode(current.manifest_bytes())?;
    fs::remove_file(root_pool_path(sandbox.path(), &current_root))?;
    let candidate = successor_root(&current_root)?;
    let preparation = successor_preparation(&current_root, &current_manifest, candidate.encoded())?;
    let before = retention_witness(sandbox.path())?;

    let error = RetentionPublicationStorage::verify_current(&mut authority, &preparation)
        .err()
        .ok_or("successor was admitted over an absent predecessor root file")?;

    assert!(matches!(
        refusal(&error),
        Some(RetentionCurrentStateRefusal::PredecessorRootAbsent)
    ));
    assert_eq!(retention_witness(sandbox.path())?, before);
    Ok(())
}

#[test]
fn successor_refuses_when_the_predecessor_root_bytes_changed() -> Result<(), Box<dyn Error>> {
    let (sandbox, mut authority) = open_authority("filesystem-retention-predecessor-changed")?;
    let root_bytes = fixture(ROOT_HEX)?;
    let _published =
        execute_retention_publication(&mut authority, &initial_preparation(&root_bytes)?)?;
    let current = authority
        .observe_current()?
        .ok_or("published retention head was not observed")?;
    let current_root = AdmittedRetentionRoot::decode(&root_bytes)?;
    let current_manifest = AdmittedRetentionManifest::decode(current.manifest_bytes())?;
    let pool_entry = root_pool_path(sandbox.path(), &current_root);
    let mut changed = fs::read(&pool_entry)?;
    *changed.last_mut().ok_or("empty root pool entry")? ^= 0x01;
    fs::write(&pool_entry, &changed)?;
    let candidate = successor_root(&current_root)?;
    let preparation = successor_preparation(&current_root, &current_manifest, candidate.encoded())?;

    let error = RetentionPublicationStorage::verify_current(&mut authority, &preparation)
        .err()
        .ok_or("successor was admitted over changed predecessor root bytes")?;

    assert!(matches!(
        refusal(&error),
        Some(RetentionCurrentStateRefusal::PredecessorRootChanged)
    ));
    Ok(())
}

#[test]
fn predecessor_read_bound_derives_from_the_typed_root_limits() -> Result<(), Box<dyn Error>> {
    let anchors = usize::try_from(RetentionRoot::MAXIMUM_ANCHOR_COUNT)?;
    let namespace = usize::from(RetentionNamespace::MAXIMUM_BYTE_LENGTH);
    let derived = 192_usize + namespace + anchors * 119 + 64;

    assert_eq!(super::root_header_decoder::MAXIMUM_ENCODED_LENGTH, derived);
    Ok(())
}
