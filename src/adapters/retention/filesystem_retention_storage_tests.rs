//! Filesystem retention publication storage laws.

use std::collections::BTreeSet;
use std::error::Error;
use std::ffi::OsString;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use super::filesystem_retention_test_fixture::{
    HEAD_HEX, MANIFEST_HEX, ROOT_HEX, fixture, open_authority, with_snapshot,
};
use super::{
    AdmittedRetentionRoot, RetentionPublicationError, RetentionPublicationOutcome,
    RetentionPublicationPreparation, RetentionPublicationStorage, RetentionTransitionDisposition,
};
use crate::{
    RetentionGenerationExpectation, execute_retention_publication, preflight_retention_transition,
    prepare_retention_publication,
};

#[test]
fn complete_publication_preserves_migrated_bytes_and_publishes_exact_retention_prefix()
-> Result<(), Box<dyn Error>> {
    let (sandbox, mut authority) = open_authority("filesystem-retention-complete")?;
    let before = migrated_witness(sandbox.path())?;
    let root_bytes = fixture(ROOT_HEX)?;
    let preparation = publish_preparation(&root_bytes)?;

    let receipt = execute_retention_publication(&mut authority, &preparation)?;

    assert_eq!(receipt.outcome(), RetentionPublicationOutcome::Published);
    assert_eq!(migrated_witness(sandbox.path())?, before);
    assert_eq!(fs::read(head_path(sandbox.path()))?, fixture(HEAD_HEX)?);
    assert_eq!(
        fs::read(root_pool_path(sandbox.path(), &preparation))?,
        root_bytes
    );
    assert_eq!(
        fs::read(manifest_pool_path(sandbox.path(), &preparation))?,
        fixture(MANIFEST_HEX)?
    );
    assert_stages_absent(sandbox.path())?;
    drop(authority);
    sandbox.remove()?;
    Ok(())
}

#[test]
fn existing_root_stage_is_never_truncated() -> Result<(), Box<dyn Error>> {
    let (sandbox, mut authority) = open_authority("filesystem-retention-exclusive-stage")?;
    let root_bytes = fixture(ROOT_HEX)?;
    let preparation = publish_preparation(&root_bytes)?;
    let stage = sandbox.path().join("retention").join("root.next");
    fs::write(&stage, b"retained partial evidence")?;

    let error =
        RetentionPublicationStorage::write_root_stage(&mut authority, preparation.candidate())
            .err()
            .ok_or("existing root stage was unexpectedly replaced")?;

    assert_eq!(error.kind(), io::ErrorKind::AlreadyExists);
    assert_eq!(fs::read(&stage)?, b"retained partial evidence");
    drop(authority);
    sandbox.remove()?;
    Ok(())
}

#[test]
fn retained_stage_refuses_publication_before_recovery() -> Result<(), Box<dyn Error>> {
    let (sandbox, mut authority) = open_authority("filesystem-retention-recovery-required")?;
    let root_bytes = fixture(ROOT_HEX)?;
    let preparation = publish_preparation(&root_bytes)?;
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
    assert!(!head_path(sandbox.path()).exists());
    drop(authority);
    sandbox.remove()?;
    Ok(())
}

#[test]
fn byte_equal_substituted_canonical_root_is_refused() -> Result<(), Box<dyn Error>> {
    let (sandbox, mut authority) = open_authority("filesystem-retention-substituted-target")?;
    let root_bytes = fixture(ROOT_HEX)?;
    let preparation = publish_preparation(&root_bytes)?;
    let candidate = preparation.candidate();
    let _disposition = RetentionPublicationStorage::verify_current(&mut authority, &preparation)?;
    RetentionPublicationStorage::write_root_stage(&mut authority, candidate)?;
    RetentionPublicationStorage::synchronize_root_stage(&mut authority)?;
    let _admission = RetentionPublicationStorage::admit_root_namespace(&mut authority, candidate)?;
    RetentionPublicationStorage::synchronize_roots_after_namespace(&mut authority)?;
    fs::write(root_pool_path(sandbox.path(), &preparation), &root_bytes)?;

    let error = RetentionPublicationStorage::link_root(&mut authority, candidate)
        .err()
        .ok_or("byte-equal substituted root was unexpectedly admitted")?;

    assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    drop(authority);
    sandbox.remove()?;
    Ok(())
}

#[test]
fn exact_committed_retry_mutates_nothing() -> Result<(), Box<dyn Error>> {
    let (sandbox, mut authority) = open_authority("filesystem-retention-exact-retry")?;
    let root_bytes = fixture(ROOT_HEX)?;
    let preparation = publish_preparation(&root_bytes)?;
    let _published = execute_retention_publication(&mut authority, &preparation)?;
    let after_publication = retention_witness(sandbox.path())?;
    let retry = publish_preparation(&root_bytes)?;

    let receipt = execute_retention_publication(&mut authority, &retry)?;

    assert_eq!(
        receipt.outcome(),
        RetentionPublicationOutcome::AlreadyCommitted
    );
    assert_eq!(retention_witness(sandbox.path())?, after_publication);
    drop(authority);
    sandbox.remove()?;
    Ok(())
}

fn publish_preparation(
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
    let preparation = prepare_retention_publication(preflight, None)?;
    assert_eq!(
        preparation.disposition(),
        RetentionTransitionDisposition::Publish
    );
    Ok(preparation)
}

fn head_path(root: &Path) -> PathBuf {
    root.join("retention").join("HEAD")
}

fn root_pool_path(root: &Path, preparation: &RetentionPublicationPreparation<'_>) -> PathBuf {
    let candidate = preparation.candidate();
    root.join("retention")
        .join("roots")
        .join(hex(candidate.root().namespace().digest().as_bytes()))
        .join(format!(
            "{:016x}-{}.root",
            candidate.root().generation().get(),
            hex(candidate.digest().as_bytes())
        ))
}

fn manifest_pool_path(root: &Path, preparation: &RetentionPublicationPreparation<'_>) -> PathBuf {
    root.join("retention").join("manifests").join(format!(
        "{:016x}-{}.manifest",
        preparation.liveness_generation().get(),
        hex(preparation.manifest_digest().as_bytes())
    ))
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

fn retention_witness(root: &Path) -> io::Result<BTreeSet<(OsString, Vec<u8>)>> {
    let mut witness = BTreeSet::new();
    collect(&root.join("retention"), &mut witness)?;
    Ok(witness)
}

fn collect(directory: &Path, witness: &mut BTreeSet<(OsString, Vec<u8>)>) -> io::Result<()> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        if entry.file_type()?.is_dir() {
            collect(&path, witness)?;
        } else {
            witness.insert((path.into_os_string(), fs::read(entry.path())?));
        }
    }
    Ok(())
}

fn hex(bytes: &[u8; 32]) -> String {
    use std::fmt::Write as _;
    bytes.iter().fold(String::new(), |mut rendered, byte| {
        let _ = write!(rendered, "{byte:02x}");
        rendered
    })
}
