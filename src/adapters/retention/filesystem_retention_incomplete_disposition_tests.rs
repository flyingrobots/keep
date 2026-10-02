//! This module owns preservation of incomplete stages under the bounded landing contract.

use super::filesystem_retention_test_fixture::{
    HEAD_HEX, MANIFEST_HEX, ROOT_HEX, drive_publication, fixture, initial_preparation,
    open_authority, retention_witness,
};
use super::{
    FilesystemRetentionRecoveryError, RetentionCurrentStateRefusal, RetentionFixedStage as Stage,
    RetentionPublicationStorage, RetentionRecoveryRefusal,
};
use std::{error::Error, fs};

// Size: medium. Oracle: maintainer decision A forbids mutation before incomplete disposition.
// Behavior change: former incomplete-stage cleanup successes now refuse; delete only if superseded.
#[test]
fn incomplete_stages_block_direct_and_publication_recovery_without_effects()
-> Result<(), Box<dyn Error>> {
    for (stage, name, corpus) in [
        (Stage::Root, "root.next", ROOT_HEX),
        (Stage::Manifest, "manifest.next", MANIFEST_HEX),
        (Stage::Head, "head.next", HEAD_HEX),
    ] {
        for length in [0_usize, 20] {
            let (sandbox, mut authority) =
                open_authority(&format!("incomplete-{stage:?}-{length}"))?;
            let root = fixture(ROOT_HEX)?;
            let preparation = initial_preparation(&root)?;
            // Leave the earlier root complete but unlinked. Even linking it is forbidden.
            drive_publication(&mut authority, &preparation, 2)?;
            fs::write(
                sandbox.path().join("retention").join(name),
                fixture(corpus)?.get(..length).ok_or("short corpus")?,
            )?;
            let before = retention_witness(sandbox.path())?;
            let direct = authority.recover();
            assert!(
                matches!(direct, Err(FilesystemRetentionRecoveryError::Plan { source: RetentionRecoveryRefusal::IncompleteStageRequiresDisposition { stage: actual, observed, .. } }) if actual == stage && observed == length),
                "direct incomplete refusal must identify observed stage: {direct:?}"
            );
            assert_eq!(
                retention_witness(sandbox.path())?,
                before,
                "direct refusal must initiate no namespace effects"
            );
            let error = authority
                .verify_current(&preparation)
                .err()
                .ok_or("publication accepted incomplete stage")?;
            assert!(
                matches!(error.get_ref().and_then(|source| source.downcast_ref::<RetentionCurrentStateRefusal>()), Some(RetentionCurrentStateRefusal::RecoveryRefused { source: RetentionRecoveryRefusal::IncompleteStageRequiresDisposition { stage: actual, observed, .. } }) if *actual == stage && *observed == length),
                "publication must preserve typed disposition requirement: {error:?}"
            );
            assert_eq!(
                retention_witness(sandbox.path())?,
                before,
                "publication refusal must initiate no namespace effects"
            );
        }
    }
    Ok(())
}

// Size: medium. Oracle: unknown completion feasibility never authorizes destruction.
// Delete only when stronger recovery-boundary preservation covers this permanent counterexample.
#[test]
fn greatest_namespace_with_a_declared_successor_preserves_the_incomplete_manifest()
-> Result<(), Box<dyn Error>> {
    let (sandbox, mut authority) = open_authority("incomplete-max-namespace")?;
    let root = fixture(ROOT_HEX)?;
    let preparation = initial_preparation(&root)?;
    drive_publication(&mut authority, &preparation, 7)?;
    let mut manifest = fixture(MANIFEST_HEX)?;
    manifest
        .get_mut(24..32)
        .ok_or("length absent")?
        .copy_from_slice(&368_u64.to_be_bytes());
    manifest
        .get_mut(44..48)
        .ok_or("count absent")?
        .copy_from_slice(&2_u32.to_be_bytes());
    manifest
        .get_mut(160..192)
        .ok_or("namespace absent")?
        .fill(u8::MAX);
    manifest.truncate(232);
    fs::write(sandbox.path().join("retention/manifest.next"), manifest)?;
    let before = retention_witness(sandbox.path())?;
    let result = authority.recover();
    assert!(
        matches!(
            result,
            Err(FilesystemRetentionRecoveryError::Plan {
                source: RetentionRecoveryRefusal::IncompleteStageRequiresDisposition {
                    stage: Stage::Manifest,
                    expected: 368,
                    observed: 232
                }
            })
        ),
        "incomplete manifest must require disposition without claiming canonical completion: {result:?}"
    );
    assert_eq!(
        retention_witness(sandbox.path())?,
        before,
        "unknown completion must preserve evidence"
    );
    Ok(())
}
