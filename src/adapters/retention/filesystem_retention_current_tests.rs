//! Filesystem retention current-state laws: a committed claim is reopened, never inferred.

use std::error::Error;
use std::fs;
use std::io;

use super::filesystem_retention_test_fixture::{
    ROOT_HEX, fixture, head_path, initial_preparation, manifest_pool_path, open_authority, refusal,
    retention_witness, root_pool_path, successor_preparation, successor_root,
};
use super::{
    AdmittedRetentionManifest, AdmittedRetentionRoot, CanonicalRetentionHead,
    RetentionCurrentStateRefusal, RetentionPublicationError,
};
use crate::{RetentionHead, RetentionManifestLength, execute_retention_publication};

#[test]
fn committed_retry_refuses_when_the_selected_root_is_absent() -> Result<(), Box<dyn Error>> {
    let (sandbox, mut authority) = open_authority("filesystem-retention-committed-root-absent")?;
    let root_bytes = fixture(ROOT_HEX)?;
    let published = initial_preparation(&root_bytes)?;
    let _receipt = execute_retention_publication(&mut authority, &published)?;
    fs::remove_file(root_pool_path(sandbox.path(), published.candidate()))?;
    let before = retention_witness(sandbox.path())?;
    let retry = initial_preparation(&root_bytes)?;

    let error = execute_retention_publication(&mut authority, &retry)
        .err()
        .ok_or("committed retry succeeded although its root pool entry is absent")?;

    let RetentionPublicationError::CurrentVerification { source } = error else {
        return Err("absent root refused outside current-state verification".into());
    };
    assert_eq!(source.kind(), io::ErrorKind::InvalidData);
    assert!(matches!(
        refusal(&source),
        Some(RetentionCurrentStateRefusal::CommittedRootAbsent)
    ));
    assert_eq!(retention_witness(sandbox.path())?, before);
    Ok(())
}

#[test]
fn committed_retry_refuses_when_the_selected_root_bytes_changed() -> Result<(), Box<dyn Error>> {
    let (sandbox, mut authority) = open_authority("filesystem-retention-committed-root-changed")?;
    let root_bytes = fixture(ROOT_HEX)?;
    let published = initial_preparation(&root_bytes)?;
    let _receipt = execute_retention_publication(&mut authority, &published)?;
    let pool_entry = root_pool_path(sandbox.path(), published.candidate());
    let mut changed = fs::read(&pool_entry)?;
    *changed.last_mut().ok_or("empty root pool entry")? ^= 0x01;
    fs::write(&pool_entry, &changed)?;
    let retry = initial_preparation(&root_bytes)?;

    let error = execute_retention_publication(&mut authority, &retry)
        .err()
        .ok_or("committed retry succeeded although its root pool bytes changed")?;

    let RetentionPublicationError::CurrentVerification { source } = error else {
        return Err("changed root refused outside current-state verification".into());
    };
    assert_eq!(source.kind(), io::ErrorKind::InvalidData);
    assert!(matches!(
        refusal(&source),
        Some(RetentionCurrentStateRefusal::CommittedRootChanged)
    ));
    Ok(())
}

#[test]
fn committed_retry_refuses_when_the_selected_manifest_is_corrupt() -> Result<(), Box<dyn Error>> {
    let (sandbox, mut authority) =
        open_authority("filesystem-retention-committed-manifest-corrupt")?;
    let root_bytes = fixture(ROOT_HEX)?;
    let published = initial_preparation(&root_bytes)?;
    let _receipt = execute_retention_publication(&mut authority, &published)?;
    let pool_entry = manifest_pool_path(sandbox.path(), &published);
    let mut corrupt = fs::read(&pool_entry)?;
    *corrupt.last_mut().ok_or("empty manifest pool entry")? ^= 0x01;
    fs::write(&pool_entry, &corrupt)?;
    let retry = initial_preparation(&root_bytes)?;

    let error = execute_retention_publication(&mut authority, &retry)
        .err()
        .ok_or("committed retry succeeded although its manifest is corrupt")?;

    assert!(matches!(
        error,
        RetentionPublicationError::CurrentVerification { .. }
    ));
    Ok(())
}

#[test]
fn head_predecessor_disagreeing_with_its_manifest_refuses() -> Result<(), Box<dyn Error>> {
    let (sandbox, mut authority) = open_authority("filesystem-retention-predecessor-disagrees")?;
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
    let advanced = authority
        .observe_current()?
        .ok_or("advanced retention head was not observed")?;
    let advanced_manifest = AdmittedRetentionManifest::decode(advanced.manifest_bytes())?;
    let wrong_predecessor = advanced_manifest.digest();
    let inconsistent = RetentionHead::new(
        advanced_manifest.manifest().generation(),
        RetentionManifestLength::new(u64::try_from(advanced.manifest_bytes().len())?)?,
        advanced_manifest.digest(),
        Some(wrong_predecessor),
    )?;
    fs::write(
        head_path(sandbox.path()),
        CanonicalRetentionHead::from_head(&inconsistent).encoded(),
    )?;

    let error = authority
        .observe_current()
        .err()
        .ok_or("head with a disagreeing predecessor was unexpectedly observed")?;

    assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    assert!(matches!(
        refusal(&error),
        Some(RetentionCurrentStateRefusal::HeadPredecessorDisagreed)
    ));
    Ok(())
}
