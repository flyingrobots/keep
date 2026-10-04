//! These laws own selected predecessor admission before recovery effects.
//! Size: medium. Oracle: recovery must reopen the exact manifest-selected root.
//! Delete only with the protocol or a stronger public-boundary replacement.

use std::error::Error;
use std::fs;

use super::filesystem_retention_test_fixture::{
    ROOT_HEX, drive_publication, fixture, initial_preparation, initial_root, open_authority,
    refusal, retention_witness, root_pool_path, successor_preparation, successor_root,
};
use super::{
    AdmittedRetentionManifest, AdmittedRetentionRoot, FilesystemRetentionRecoveryError,
    RetentionCurrentStateRefusal,
};
use crate::execute_retention_publication;

#[derive(Clone, Copy, Debug)]
enum Damage {
    Missing,
    Corrupt,
    Substituted,
}

#[test]
fn recovery_refuses_a_missing_predecessor_before_root_link() -> Result<(), Box<dyn Error>> {
    require_refusal(Damage::Missing, 2)
}

#[test]
fn recovery_refuses_a_corrupt_predecessor_before_root_link() -> Result<(), Box<dyn Error>> {
    require_refusal(Damage::Corrupt, 2)
}

#[test]
fn recovery_refuses_a_substituted_predecessor_before_root_link() -> Result<(), Box<dyn Error>> {
    require_refusal(Damage::Substituted, 2)
}

#[test]
fn recovery_refuses_a_missing_predecessor_before_manifest_link() -> Result<(), Box<dyn Error>> {
    require_refusal(Damage::Missing, 8)
}

#[test]
fn recovery_refuses_a_corrupt_predecessor_before_manifest_link() -> Result<(), Box<dyn Error>> {
    require_refusal(Damage::Corrupt, 8)
}

#[test]
fn recovery_refuses_a_substituted_predecessor_before_manifest_link() -> Result<(), Box<dyn Error>> {
    require_refusal(Damage::Substituted, 8)
}

#[test]
fn recovery_refuses_a_missing_predecessor_before_head_commit() -> Result<(), Box<dyn Error>> {
    require_refusal(Damage::Missing, 13)
}

#[test]
fn recovery_refuses_a_corrupt_predecessor_before_head_commit() -> Result<(), Box<dyn Error>> {
    require_refusal(Damage::Corrupt, 13)
}

#[test]
fn recovery_refuses_a_substituted_predecessor_before_head_commit() -> Result<(), Box<dyn Error>> {
    require_refusal(Damage::Substituted, 13)
}

fn require_refusal(damage: Damage, prefix: usize) -> Result<(), Box<dyn Error>> {
    // Deterministic fault matrix: root link, manifest link, head replacement.
    let label = format!("recovery-predecessor-{damage:?}-{prefix}");
    let (sandbox, mut authority) = open_authority(&label)?;
    let root = fixture(ROOT_HEX)?;
    let admitted = AdmittedRetentionRoot::decode(&root)?;
    let _published = execute_retention_publication(&mut authority, &initial_preparation(&root)?)?;
    let current = authority
        .observe_current()?
        .ok_or("missing published state")?;
    let manifest = AdmittedRetentionManifest::decode(current.manifest_bytes())?;
    let successor = successor_root(&admitted)?;
    let preparation = successor_preparation(&admitted, &manifest, successor.encoded())?;
    drive_publication(&mut authority, &preparation, prefix)?;
    let selected = root_pool_path(sandbox.path(), &admitted);
    damage_predecessor(&selected, &admitted, damage)?;
    let before = retention_witness(sandbox.path())?;

    let result = authority.recover();

    assert_eq!(
        retention_witness(sandbox.path())?,
        before,
        "selected predecessor refusal must preserve retained bytes for {label}: {result:?}"
    );
    let expected = match (&result, damage) {
        (Err(FilesystemRetentionRecoveryError::Observe { source }), Damage::Missing) => {
            matches!(
                refusal(source),
                Some(RetentionCurrentStateRefusal::PredecessorRootAbsent)
            )
        }
        (Err(FilesystemRetentionRecoveryError::Observe { source }), _) => {
            matches!(
                refusal(source),
                Some(RetentionCurrentStateRefusal::PredecessorRootChanged)
            )
        }
        _ => false,
    };
    assert!(
        expected,
        "selected predecessor must refuse at observation for {label}: {result:?}"
    );
    Ok(())
}

fn damage_predecessor(
    path: &std::path::Path,
    root: &AdmittedRetentionRoot<'_>,
    damage: Damage,
) -> Result<(), Box<dyn Error>> {
    match damage {
        Damage::Missing => fs::remove_file(path)?,
        Damage::Corrupt => {
            let mut bytes = root.encoded().to_vec();
            let checksum = bytes.last_mut().ok_or("missing root checksum")?;
            *checksum ^= 1;
            fs::write(path, bytes)?;
        }
        Damage::Substituted => {
            let replacement = initial_root(b"replacement", root)?;
            fs::write(path, replacement.encoded())?;
        }
    }
    Ok(())
}
