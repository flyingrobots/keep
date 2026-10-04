//! These laws own a bounded candidate-generation domain at filesystem recovery.

use std::error::Error;

use super::filesystem_retention_recovery_history_tests::install_history;
use super::filesystem_retention_test_fixture::{
    ROOT_HEX, drive_publication, fixture, initial_preparation, open_authority, retention_witness,
    successor_preparation, successor_root,
};
use super::{
    AdmittedRetentionManifest, AdmittedRetentionRoot, CanonicalRetentionRoot,
    FilesystemRetentionRecoveryError, RetentionRecoveryRefusal,
};
use crate::{RetentionPolicy, RetentionRoot, RootGeneration, execute_retention_publication};

// Size: medium. Oracle: generation one admits only candidate generation two.
// Deterministic domain: 3..=16 plus u64::MAX, with ascending minimal counterexample.
// Replay: cargo test --lib --all-features recovered_heads_refuse_the_skipped_generation_domain
// Delete only with this protocol or a stronger runtime boundary replacement.
#[test]
fn recovered_heads_refuse_the_skipped_generation_domain() -> Result<(), Box<dyn Error>> {
    for generation in (3..=16).chain([u64::MAX]) {
        require_generation_refusal(generation)?;
    }
    Ok(())
}

fn require_generation_refusal(generation: u64) -> Result<(), Box<dyn Error>> {
    let (sandbox, mut authority) = open_authority("recovery-skipped-generation-domain")?;
    let bytes = fixture(ROOT_HEX)?;
    let initial = AdmittedRetentionRoot::decode(&bytes)?;
    let _published = execute_retention_publication(&mut authority, &initial_preparation(&bytes)?)?;
    let current = authority
        .observe_current()?
        .ok_or("missing current state")?;
    let manifest = AdmittedRetentionManifest::decode(current.manifest_bytes())?;
    let successor = successor_root(&initial)?;
    let preparation = successor_preparation(&initial, &manifest, successor.encoded())?;
    drive_publication(&mut authority, &preparation, 13)?;
    let skipped = RetentionRoot::new(
        initial.root().namespace().clone(),
        RootGeneration::new(generation)?,
        RetentionPolicy::new(initial.root().profile(), initial.root().limits()),
        Some(initial.digest()),
        initial.root().anchors().to_vec(),
    )?;
    let encoded = CanonicalRetentionRoot::from_root(&skipped)?;
    install_history(sandbox.path(), &preparation, &encoded)?;
    let before = retention_witness(sandbox.path())?;
    let result = authority.recover();
    assert_eq!(
        retention_witness(sandbox.path())?,
        before,
        "candidate generation {generation} must preserve every retained byte: {result:?}"
    );
    assert!(
        matches!(
            result,
            Err(FilesystemRetentionRecoveryError::Plan {
                source: RetentionRecoveryRefusal::RootNotSuccessor
            })
        ),
        "candidate generation {generation} must refuse with RootNotSuccessor: {result:?}"
    );
    Ok(())
}
