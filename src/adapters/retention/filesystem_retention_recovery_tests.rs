//! Filesystem retention recovery laws over real crash prefixes.

use std::error::Error;
use std::fs;

use super::filesystem_retention_test_fixture::{
    ROOT_HEX, drive_publication, fixture, head_path, initial_preparation, manifest_pool_path,
    open_authority, root_pool_path,
};
use super::{
    RetentionPublicationOutcome, RetentionRecoveryOutcome as Outcome, RetentionRecoveryStep as Step,
};
use crate::execute_retention_publication;

#[test]
fn a_clean_store_recovers_to_clean() -> Result<(), Box<dyn Error>> {
    let (_sandbox, mut authority) = open_authority("filesystem-retention-recovery-clean")?;
    let receipt = authority.recover()?;
    assert!(receipt.executed().is_empty());
    assert_eq!(receipt.outcome(), Outcome::Clean);
    Ok(())
}

#[test]
fn a_written_root_stage_is_linked_and_protected() -> Result<(), Box<dyn Error>> {
    let (sandbox, mut authority) = open_authority("filesystem-retention-recovery-root")?;
    let root_bytes = fixture(ROOT_HEX)?;
    let preparation = initial_preparation(&root_bytes)?;
    drive_publication(&mut authority, &preparation, 3)?;

    let receipt = authority.recover()?;

    assert_eq!(receipt.executed(), [Step::LinkRoot]);
    assert_eq!(
        receipt.outcome(),
        Outcome::Protected {
            root_stage: true,
            manifest_stage: false
        }
    );
    assert_eq!(
        fs::read(root_pool_path(sandbox.path(), preparation.candidate()))?,
        root_bytes
    );
    assert!(sandbox.path().join("retention").join("root.next").is_file());
    assert!(!head_path(sandbox.path()).exists());
    Ok(())
}

#[test]
fn a_synchronized_head_stage_is_finalized_and_the_retry_is_already_committed()
-> Result<(), Box<dyn Error>> {
    let (sandbox, mut authority) = open_authority("filesystem-retention-recovery-head")?;
    let root_bytes = fixture(ROOT_HEX)?;
    let preparation = initial_preparation(&root_bytes)?;
    drive_publication(&mut authority, &preparation, 13)?;
    let publication = preparation.publication().ok_or("no publication")?;

    let receipt = authority.recover()?;

    assert_eq!(
        receipt.executed(),
        [
            Step::FinalizeHead,
            Step::RemoveRootStage,
            Step::RemoveManifestStage
        ]
    );
    assert_eq!(receipt.outcome(), Outcome::Committed);
    assert_eq!(
        fs::read(head_path(sandbox.path()))?,
        publication.head().encoded()
    );
    assert_eq!(
        fs::read(manifest_pool_path(sandbox.path(), &preparation))?,
        publication.manifest().encoded()
    );
    for stage in ["root.next", "manifest.next", "head.next"] {
        assert!(
            !sandbox.path().join("retention").join(stage).exists(),
            "{stage} remained"
        );
    }
    let retry = execute_retention_publication(&mut authority, &preparation)?;
    assert_eq!(
        retry.outcome(),
        RetentionPublicationOutcome::AlreadyCommitted
    );
    Ok(())
}

#[test]
fn a_truncated_root_stage_is_discarded() -> Result<(), Box<dyn Error>> {
    let (sandbox, mut authority) = open_authority("filesystem-retention-recovery-truncated")?;
    let root_bytes = fixture(ROOT_HEX)?;
    let stage = sandbox.path().join("retention").join("root.next");
    fs::write(
        &stage,
        root_bytes
            .get(..100)
            .ok_or("root fixture shorter than 100 bytes")?,
    )?;

    let receipt = authority.recover()?;

    assert_eq!(receipt.executed(), [Step::DiscardRootStage]);
    assert_eq!(receipt.outcome(), Outcome::Clean);
    assert!(!stage.exists());
    Ok(())
}
