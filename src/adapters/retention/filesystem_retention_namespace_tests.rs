//! Filesystem retention namespace laws: unknown entries are ambiguity, not noise.

use std::error::Error;
use std::fs;
use std::io;

use super::RetentionPublicationStorage;
use super::filesystem_retention_test_fixture::{
    ROOT_HEX, fixture, initial_preparation, open_authority, retention_witness,
};

#[test]
fn unknown_retention_entry_refuses_before_any_stage_is_written() -> Result<(), Box<dyn Error>> {
    let (sandbox, mut authority) = open_authority("filesystem-retention-unknown-entry")?;
    fs::write(
        sandbox.path().join("retention").join("junk"),
        b"not protocol state",
    )?;
    let before = retention_witness(sandbox.path())?;
    let root_bytes = fixture(ROOT_HEX)?;
    let preparation = initial_preparation(&root_bytes)?;

    let error = RetentionPublicationStorage::verify_current(&mut authority, &preparation)
        .err()
        .ok_or("unknown retention entry was unexpectedly admitted")?;

    assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    assert!(matches!(
        super::filesystem_retention_test_fixture::refusal(&error),
        Some(super::RetentionCurrentStateRefusal::UnknownRetentionEntry)
    ));
    assert_eq!(retention_witness(sandbox.path())?, before);
    drop(authority);
    sandbox.remove()?;
    Ok(())
}

#[test]
fn non_digest_root_namespace_directory_refuses() -> Result<(), Box<dyn Error>> {
    let (sandbox, mut authority) = open_authority("filesystem-retention-bad-namespace-name")?;
    fs::create_dir(
        sandbox
            .path()
            .join("retention")
            .join("roots")
            .join("not-a-digest"),
    )?;
    let root_bytes = fixture(ROOT_HEX)?;
    let preparation = initial_preparation(&root_bytes)?;

    let error = RetentionPublicationStorage::verify_current(&mut authority, &preparation)
        .err()
        .ok_or("non-digest namespace directory was unexpectedly admitted")?;

    assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    assert!(matches!(
        super::filesystem_retention_test_fixture::refusal(&error),
        Some(super::RetentionCurrentStateRefusal::NonNamespaceEntry)
    ));
    drop(authority);
    sandbox.remove()?;
    Ok(())
}

#[test]
fn malformed_manifest_pool_name_refuses() -> Result<(), Box<dyn Error>> {
    let (sandbox, mut authority) = open_authority("filesystem-retention-bad-manifest-name")?;
    fs::write(
        sandbox
            .path()
            .join("retention")
            .join("manifests")
            .join("bogus.manifest"),
        b"",
    )?;
    let root_bytes = fixture(ROOT_HEX)?;
    let preparation = initial_preparation(&root_bytes)?;

    let error = RetentionPublicationStorage::verify_current(&mut authority, &preparation)
        .err()
        .ok_or("malformed manifest pool name was unexpectedly admitted")?;

    assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    assert!(matches!(
        super::filesystem_retention_test_fixture::refusal(&error),
        Some(super::RetentionCurrentStateRefusal::NoncanonicalPoolEntry { .. })
    ));
    drop(authority);
    sandbox.remove()?;
    Ok(())
}

#[test]
fn uppercase_root_pool_name_refuses() -> Result<(), Box<dyn Error>> {
    let (sandbox, mut authority) = open_authority("filesystem-retention-uppercase-root-name")?;
    let namespace = sandbox
        .path()
        .join("retention")
        .join("roots")
        .join("a".repeat(64));
    fs::create_dir(&namespace)?;
    fs::write(
        namespace.join(format!("{}-{}.root", "0".repeat(15) + "1", "A".repeat(64))),
        b"",
    )?;
    let root_bytes = fixture(ROOT_HEX)?;
    let preparation = initial_preparation(&root_bytes)?;

    let error = RetentionPublicationStorage::verify_current(&mut authority, &preparation)
        .err()
        .ok_or("uppercase root pool name was unexpectedly admitted")?;

    assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    assert!(matches!(
        super::filesystem_retention_test_fixture::refusal(&error),
        Some(super::RetentionCurrentStateRefusal::NoncanonicalPoolEntry { .. })
    ));
    drop(authority);
    sandbox.remove()?;
    Ok(())
}
