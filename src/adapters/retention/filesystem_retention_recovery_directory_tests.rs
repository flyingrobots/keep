//! These laws own directory-capability binding before explicit recovery.

use std::error::Error;
use std::fs;
use std::io;

use super::filesystem_retention_test_fixture::{ROOT_HEX, fixture, open_authority, refusal};
use super::{FilesystemRetentionRecoveryError, RetentionCurrentStateRefusal};

#[test]
fn replaced_retention_directory_refuses_recovery_without_deleting_evidence()
-> Result<(), Box<dyn Error>> {
    assert_replacement_refuses("retention")
}

#[test]
fn replaced_root_pool_refuses_recovery_without_deleting_evidence() -> Result<(), Box<dyn Error>> {
    assert_replacement_refuses("retention/roots")
}

#[test]
fn replaced_manifest_pool_refuses_recovery_without_deleting_evidence() -> Result<(), Box<dyn Error>>
{
    assert_replacement_refuses("retention/manifests")
}

fn assert_replacement_refuses(name: &str) -> Result<(), Box<dyn Error>> {
    let fixture_name = format!("recovery-directory-binding-{}", name.replace('/', "-"));
    let (sandbox, mut authority) = open_authority(&fixture_name)?;
    let bytes = fixture(ROOT_HEX)?;
    let prefix = bytes.get(..100).ok_or("root fixture too short")?;
    let retention = sandbox.path().join("retention");
    let stage = retention.join("root.next");
    fs::write(&stage, prefix)?;
    let replaced = sandbox.path().join(name);
    let archived = replaced.with_extension("previous");
    fs::rename(&replaced, &archived)?;
    fs::create_dir(&replaced)?;
    let retained_stage = if name == "retention" {
        fs::create_dir(replaced.join("roots"))?;
        fs::create_dir(replaced.join("manifests"))?;
        archived.join("root.next")
    } else {
        stage
    };

    let error = authority
        .recover()
        .err()
        .ok_or("direct recovery mutated a replaced protocol directory")?;
    assert_directory_refusal(error)?;
    assert_eq!(fs::read(retained_stage)?, prefix);
    assert!(!retention.join("HEAD").exists());
    Ok(())
}

fn assert_directory_refusal(error: FilesystemRetentionRecoveryError) -> Result<(), Box<dyn Error>> {
    let FilesystemRetentionRecoveryError::Observe { source } = error else {
        return Err("directory replacement refused outside observation".into());
    };
    assert_eq!(source.kind(), io::ErrorKind::InvalidData);
    assert!(matches!(
        refusal(&source),
        Some(RetentionCurrentStateRefusal::ProtocolDirectoryReplaced)
    ));
    Ok(())
}
