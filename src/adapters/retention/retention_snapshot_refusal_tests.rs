//! Selected-root refusals retain the precise bound and manifest coordinates.

use std::error::Error;
use std::fs::{self, OpenOptions};

use super::filesystem_retention_test_fixture::{
    ROOT_HEX, catalog_policy, fixture, initial_preparation, open_authority, root_pool_path,
};
use crate::{
    AdmittedRetentionRoot, CanonicalRetentionRoot, FilesystemRetentionSnapshot,
    FilesystemRetentionSnapshotError, ReaderAttemptLimit, RetentionPolicy, RetentionRoot,
    RetentionSelectedRootRefusal,
};

#[test]
fn an_oversized_selected_root_reports_the_exact_allocation_bound() -> Result<(), Box<dyn Error>> {
    let (sandbox, mut authority) = open_authority("snapshot-root-bound-source")?;
    let bytes = fixture(ROOT_HEX)?;
    let selected = AdmittedRetentionRoot::decode(&bytes)?;
    let preparation = initial_preparation(&bytes)?;
    let _receipt = crate::execute_retention_publication(&mut authority, &preparation)?;
    drop(authority);
    let snapshot = FilesystemRetentionSnapshot::load(
        sandbox.path(),
        catalog_policy()?,
        ReaderAttemptLimit::DEFAULT,
    )?;
    let head = fs::read(sandbox.path().join("retention/HEAD"))?;
    let maximum = u64::try_from(super::root_header_decoder::MAXIMUM_ENCODED_LENGTH)?;
    let observed = maximum.checked_add(1).ok_or("bound overflow")?;
    OpenOptions::new()
        .write(true)
        .open(root_pool_path(sandbox.path(), &selected))?
        .set_len(observed)?;
    let error = snapshot
        .retained_root(selected.root().namespace().digest())
        .err()
        .ok_or("oversized root admitted")?;
    let source = root_source(error)?;
    assert!(matches!(
        source.get_ref().and_then(|error| error.downcast_ref::<RetentionSelectedRootRefusal>()),
        Some(RetentionSelectedRootRefusal::Length { maximum: limit, observed: length })
            if *limit == maximum && *length == observed
    ));
    assert_eq!(fs::read(sandbox.path().join("retention/HEAD"))?, head);
    drop(snapshot);
    sandbox.remove()?;
    Ok(())
}

#[test]
fn a_valid_successor_at_the_selected_name_reports_both_selections() -> Result<(), Box<dyn Error>> {
    let (sandbox, mut authority) = open_authority("snapshot-root-selection-source")?;
    let bytes = fixture(ROOT_HEX)?;
    let selected = AdmittedRetentionRoot::decode(&bytes)?;
    let preparation = initial_preparation(&bytes)?;
    let _receipt = crate::execute_retention_publication(&mut authority, &preparation)?;
    drop(authority);
    let successor = RetentionRoot::new(
        selected.root().namespace().clone(),
        selected.root().generation().successor()?,
        RetentionPolicy::new(selected.root().profile(), selected.root().limits()),
        Some(selected.digest()),
        selected.root().anchors().to_vec(),
    )?;
    let encoded = CanonicalRetentionRoot::from_root(&successor)?;
    let snapshot = FilesystemRetentionSnapshot::load(
        sandbox.path(),
        catalog_policy()?,
        ReaderAttemptLimit::DEFAULT,
    )?;
    let head = fs::read(sandbox.path().join("retention/HEAD"))?;
    fs::write(root_pool_path(sandbox.path(), &selected), encoded.encoded())?;
    let error = snapshot
        .retained_root(selected.root().namespace().digest())
        .err()
        .ok_or("different valid root admitted under the selected name")?;
    let source = root_source(error)?;
    assert!(matches!(
        source.get_ref().and_then(|error| error.downcast_ref::<RetentionSelectedRootRefusal>()),
        Some(RetentionSelectedRootRefusal::Coordinate {
            expected_generation, observed_generation, expected_digest, observed_digest,
        }) if *expected_generation == selected.root().generation()
            && *observed_generation == successor.generation()
            && *expected_digest == selected.digest() && *observed_digest == encoded.digest()
    ));
    assert_eq!(fs::read(sandbox.path().join("retention/HEAD"))?, head);
    drop(snapshot);
    sandbox.remove()?;
    Ok(())
}

fn root_source(error: FilesystemRetentionSnapshotError) -> Result<std::io::Error, Box<dyn Error>> {
    match error {
        FilesystemRetentionSnapshotError::Root { source } => Ok(source),
        error => Err(Box::new(error)),
    }
}
