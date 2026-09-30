//! Filesystem retention publication storage laws.

use std::error::Error;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use super::filesystem_retention_test_fixture::{
    HEAD_HEX, MANIFEST_HEX, ROOT_HEX, fixture, head_path, initial_preparation, manifest_pool_path,
    open_authority, retention_witness, root_pool_path,
};
use super::{RetentionPublicationError, RetentionPublicationOutcome, RetentionPublicationStorage};
use crate::execute_retention_publication;

#[test]
fn complete_publication_preserves_migrated_bytes_and_publishes_exact_retention_prefix()
-> Result<(), Box<dyn Error>> {
    let (sandbox, mut authority) = open_authority("filesystem-retention-complete")?;
    let before = migrated_witness(sandbox.path())?;
    let root_bytes = fixture(ROOT_HEX)?;
    let preparation = initial_preparation(&root_bytes)?;

    let receipt = execute_retention_publication(&mut authority, &preparation)?;

    assert_eq!(receipt.outcome(), RetentionPublicationOutcome::Published);
    assert_eq!(migrated_witness(sandbox.path())?, before);
    assert_eq!(fs::read(head_path(sandbox.path()))?, fixture(HEAD_HEX)?);
    assert_eq!(
        fs::read(root_pool_path(sandbox.path(), preparation.candidate()))?,
        root_bytes
    );
    assert_eq!(
        fs::read(manifest_pool_path(sandbox.path(), &preparation))?,
        fixture(MANIFEST_HEX)?
    );
    assert_stages_absent(sandbox.path())?;
    Ok(())
}

#[test]
fn existing_root_stage_is_never_truncated() -> Result<(), Box<dyn Error>> {
    let (sandbox, mut authority) = open_authority("filesystem-retention-exclusive-stage")?;
    let root_bytes = fixture(ROOT_HEX)?;
    let preparation = initial_preparation(&root_bytes)?;
    assert_eq!(
        authority.verify_current(&preparation)?,
        super::RetentionTransitionDisposition::Publish
    );
    let stage = sandbox.path().join("retention").join("root.next");
    fs::write(&stage, b"retained partial evidence")?;

    let error =
        RetentionPublicationStorage::write_root_stage(&mut authority, preparation.candidate())
            .err()
            .ok_or("existing root stage was unexpectedly replaced")?;

    assert_eq!(error.kind(), io::ErrorKind::AlreadyExists);
    assert_eq!(fs::read(&stage)?, b"retained partial evidence");
    Ok(())
}

#[test]
fn retained_stage_refuses_publication_before_recovery() -> Result<(), Box<dyn Error>> {
    let (sandbox, mut authority) = open_authority("filesystem-retention-recovery-required")?;
    let root_bytes = fixture(ROOT_HEX)?;
    let preparation = initial_preparation(&root_bytes)?;
    fs::write(
        sandbox.path().join("retention").join("head.next"),
        fixture(HEAD_HEX)?,
    )?;

    let error = execute_retention_publication(&mut authority, &preparation)
        .err()
        .ok_or("retained head stage was unexpectedly published over")?;

    let RetentionPublicationError::CurrentVerification { source } = error else {
        return Err("retained stage refused outside current-state verification".into());
    };
    assert_eq!(source.kind(), io::ErrorKind::InvalidData);
    assert!(matches!(
        super::filesystem_retention_test_fixture::refusal(&source),
        Some(super::RetentionCurrentStateRefusal::RecoveryRefused { .. })
    ));
    assert!(!head_path(sandbox.path()).exists());
    Ok(())
}

#[test]
fn byte_equal_substituted_canonical_root_is_refused() -> Result<(), Box<dyn Error>> {
    let (sandbox, mut authority) = open_authority("filesystem-retention-substituted-target")?;
    let root_bytes = fixture(ROOT_HEX)?;
    let preparation = initial_preparation(&root_bytes)?;
    let candidate = preparation.candidate();
    let _disposition = RetentionPublicationStorage::verify_current(&mut authority, &preparation)?;
    RetentionPublicationStorage::write_root_stage(&mut authority, candidate)?;
    RetentionPublicationStorage::synchronize_root_stage(&mut authority)?;
    let _admission = RetentionPublicationStorage::admit_root_namespace(&mut authority, candidate)?;
    RetentionPublicationStorage::synchronize_roots_after_namespace(&mut authority)?;
    fs::write(root_pool_path(sandbox.path(), candidate), &root_bytes)?;

    let error = RetentionPublicationStorage::link_root(&mut authority, candidate)
        .err()
        .ok_or("byte-equal substituted root was unexpectedly admitted")?;

    assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    Ok(())
}

#[test]
fn exact_committed_retry_mutates_nothing() -> Result<(), Box<dyn Error>> {
    let (sandbox, mut authority) = open_authority("filesystem-retention-exact-retry")?;
    let root_bytes = fixture(ROOT_HEX)?;
    let preparation = initial_preparation(&root_bytes)?;
    let _published = execute_retention_publication(&mut authority, &preparation)?;
    let after_publication = retention_witness(sandbox.path())?;
    let retry = initial_preparation(&root_bytes)?;

    let receipt = execute_retention_publication(&mut authority, &retry)?;

    assert_eq!(
        receipt.outcome(),
        RetentionPublicationOutcome::AlreadyCommitted
    );
    assert_eq!(retention_witness(sandbox.path())?, after_publication);
    Ok(())
}

fn assert_stages_absent(root: &Path) -> Result<(), Box<dyn Error>> {
    for stage in ["root.next", "manifest.next", "head.next"] {
        let path = root.join("retention").join(stage);
        if path.exists() {
            return Err(format!("retained publication stage {stage} remained visible").into());
        }
    }
    Ok(())
}

fn migrated_witness(root: &Path) -> io::Result<Vec<(PathBuf, Vec<u8>)>> {
    let mut witness = Vec::new();
    for name in ["HEAD", "FORMAT", "migration.intent", "migration.receipt"] {
        witness.push((PathBuf::from(name), fs::read(root.join(name))?));
    }
    for pool in ["segments", "catalogs"] {
        for entry in fs::read_dir(root.join(pool))? {
            let path = entry?.path();
            let bytes = fs::read(&path)?;
            witness.push((path, bytes));
        }
    }
    witness.sort();
    Ok(witness)
}

#[test]
fn a_complete_orphan_root_stage_refuses_publication_until_disposition() -> Result<(), Box<dyn Error>>
{
    let (sandbox, mut authority) = open_authority("filesystem-retention-recovery-required-orphan")?;
    let root_bytes = fixture(ROOT_HEX)?;
    let preparation = initial_preparation(&root_bytes)?;
    fs::write(
        sandbox.path().join("retention").join("root.next"),
        &root_bytes,
    )?;

    let error = execute_retention_publication(&mut authority, &preparation)
        .err()
        .ok_or("a complete orphan root stage was unexpectedly published over")?;

    let RetentionPublicationError::CurrentVerification { source } = error else {
        return Err("protected orphan refused outside current-state verification".into());
    };
    assert!(matches!(
        super::filesystem_retention_test_fixture::refusal(&source),
        Some(super::RetentionCurrentStateRefusal::RetainedStage)
    ));
    assert!(sandbox.path().join("retention").join("root.next").is_file());
    assert_eq!(
        fs::read(root_pool_path(sandbox.path(), preparation.candidate()))?,
        root_bytes
    );
    Ok(())
}

#[test]
fn a_truncated_stage_is_recovered_and_publication_proceeds() -> Result<(), Box<dyn Error>> {
    let (sandbox, mut authority) = open_authority("filesystem-retention-recovered-stage")?;
    let root_bytes = fixture(ROOT_HEX)?;
    let preparation = initial_preparation(&root_bytes)?;
    let stage = sandbox.path().join("retention").join("manifest.next");
    fs::write(&stage, b"partial bytes left by a failed write")?;

    let receipt = execute_retention_publication(&mut authority, &preparation)?;

    assert_eq!(receipt.outcome(), RetentionPublicationOutcome::Published);
    assert!(
        !stage.exists(),
        "the truncated stage must be discarded by recovery"
    );
    assert_eq!(fs::read(head_path(sandbox.path()))?, fixture(HEAD_HEX)?);
    Ok(())
}
