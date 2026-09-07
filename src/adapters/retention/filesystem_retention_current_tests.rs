//! Filesystem retention current-state laws: a committed claim is reopened, never inferred.

use std::error::Error;
use std::fs;
use std::io;

use super::RetentionPublicationError;
use super::filesystem_retention_test_fixture::{
    ROOT_HEX, fixture, initial_preparation, manifest_pool_path, open_authority, retention_witness,
    root_pool_path,
};
use crate::execute_retention_publication;

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
    assert_eq!(retention_witness(sandbox.path())?, before);
    drop(authority);
    sandbox.remove()?;
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
    drop(authority);
    sandbox.remove()?;
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
    drop(authority);
    sandbox.remove()?;
    Ok(())
}
