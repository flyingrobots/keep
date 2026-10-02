//! These laws own refusal of incomplete stages with impossible earlier evidence.

use std::error::Error;
use std::fs;

use super::filesystem_retention_test_fixture::{
    ROOT_HEX, drive_publication, fixture, initial_preparation, manifest_pool_path, open_authority,
    retention_witness, root_pool_path,
};
use super::{FilesystemRetentionRecoveryError, RetentionFixedStage, RetentionRecoveryRefusal};

#[derive(Clone, Copy, Debug)]
enum Missing {
    RootStage,
    RootLink,
    ManifestStage,
    ManifestLink,
}

// Size: medium. Oracle: a manifest write requires a complete linked root.
// Delete only with this protocol or a stronger runtime-boundary replacement.
#[test]
fn truncated_manifest_refuses_a_missing_root_stage() -> Result<(), Box<dyn Error>> {
    require_refusal(RetentionFixedStage::Manifest, Missing::RootStage)
}
#[test]
fn truncated_manifest_refuses_a_missing_root_link() -> Result<(), Box<dyn Error>> {
    require_refusal(RetentionFixedStage::Manifest, Missing::RootLink)
}

// Size: medium. Oracle: a head write requires complete linked root and manifest.
// Delete only with this protocol or a stronger runtime-boundary replacement.
#[test]
fn truncated_head_refuses_a_missing_root_stage() -> Result<(), Box<dyn Error>> {
    require_refusal(RetentionFixedStage::Head, Missing::RootStage)
}
#[test]
fn truncated_head_refuses_a_missing_root_link() -> Result<(), Box<dyn Error>> {
    require_refusal(RetentionFixedStage::Head, Missing::RootLink)
}
#[test]
fn truncated_head_refuses_a_missing_manifest_stage() -> Result<(), Box<dyn Error>> {
    require_refusal(RetentionFixedStage::Head, Missing::ManifestStage)
}
#[test]
fn truncated_head_refuses_a_missing_manifest_link() -> Result<(), Box<dyn Error>> {
    require_refusal(RetentionFixedStage::Head, Missing::ManifestLink)
}

fn require_refusal(stage: RetentionFixedStage, missing: Missing) -> Result<(), Box<dyn Error>> {
    let label = format!("recovery-impossible-discard-{stage:?}-{missing:?}");
    let (sandbox, mut authority) = open_authority(&label)?;
    let bytes = fixture(ROOT_HEX)?;
    let preparation = initial_preparation(&bytes)?;
    let (count, name) = match stage {
        RetentionFixedStage::Manifest => (8, "manifest.next"),
        RetentionFixedStage::Head => (13, "head.next"),
        RetentionFixedStage::Root => return Err("root has no earlier stage".into()),
    };
    drive_publication(&mut authority, &preparation, count)?;
    let path = sandbox.path().join("retention").join(name);
    let complete = fs::read(&path)?;
    fs::write(
        &path,
        complete.get(..20).ok_or("stage shorter than prefix")?,
    )?;
    let earlier_stage = match missing {
        Missing::RootStage => {
            fs::remove_file(sandbox.path().join("retention/root.next"))?;
            RetentionFixedStage::Root
        }
        Missing::RootLink => {
            fs::remove_file(root_pool_path(sandbox.path(), preparation.candidate()))?;
            RetentionFixedStage::Root
        }
        Missing::ManifestStage => {
            fs::remove_file(sandbox.path().join("retention/manifest.next"))?;
            RetentionFixedStage::Manifest
        }
        Missing::ManifestLink => {
            fs::remove_file(manifest_pool_path(sandbox.path(), &preparation))?;
            RetentionFixedStage::Manifest
        }
    };
    let before = retention_witness(sandbox.path())?;
    let result = authority.recover();
    assert_eq!(
        retention_witness(sandbox.path())?,
        before,
        "impossible {stage:?} discard must preserve retained bytes: {result:?}"
    );
    assert!(
        matches!(result, Err(FilesystemRetentionRecoveryError::Plan {
        source: RetentionRecoveryRefusal::TruncatedStageWithoutEarlierEvidence {
            stage: observed, earlier_stage: earlier
        }
    }) if observed == stage && earlier == earlier_stage),
        "impossible {stage:?} prefix must name missing {earlier_stage:?}: {result:?}"
    );
    Ok(())
}
