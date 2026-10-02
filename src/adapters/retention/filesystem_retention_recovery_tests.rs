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
fn a_truncated_root_stage_is_preserved_for_disposition() -> Result<(), Box<dyn Error>> {
    let (sandbox, mut authority) = open_authority("filesystem-retention-recovery-truncated")?;
    let root_bytes = fixture(ROOT_HEX)?;
    let stage = sandbox.path().join("retention").join("root.next");
    fs::write(
        &stage,
        root_bytes
            .get(..100)
            .ok_or("root fixture shorter than 100 bytes")?,
    )?;

    let result = authority.recover();
    assert!(
        matches!(
            result,
            Err(super::FilesystemRetentionRecoveryError::Plan {
                source: super::RetentionRecoveryRefusal::IncompleteStageRequiresDisposition {
                    stage: super::RetentionFixedStage::Root,
                    expected: 192,
                    observed: 100
                }
            })
        ),
        "incomplete root must require disposition: {result:?}"
    );
    assert_eq!(
        fs::read(stage)?,
        root_bytes.get(..100).ok_or("missing prefix")?
    );
    Ok(())
}

// Size: medium. Oracle: recovery names the missing root, preserving ambiguous evidence.
// Delete only when stronger public recovery evidence subsumes this diagnostic and preservation law.
#[test]
fn a_complete_manifest_without_its_root_reports_the_missing_root() -> Result<(), Box<dyn Error>> {
    use super::filesystem_retention_test_fixture::retention_witness;
    use super::{FilesystemRetentionRecoveryError, RetentionRecoveryRefusal};

    let (sandbox, mut authority) = open_authority("recovery-missing-root")?;
    let root_bytes = fixture(ROOT_HEX)?;
    let preparation = initial_preparation(&root_bytes)?;
    drive_publication(&mut authority, &preparation, 13)?;
    fs::remove_file(sandbox.path().join("retention/root.next"))?;
    let before = retention_witness(sandbox.path())?;

    let result = authority.recover();

    assert!(
        matches!(
            result,
            Err(FilesystemRetentionRecoveryError::Plan {
                source: RetentionRecoveryRefusal::ManifestStageWithoutRootStage,
            })
        ),
        "a complete manifest with no root stage must name the missing root: {result:?}"
    );
    assert_eq!(
        retention_witness(sandbox.path())?,
        before,
        "missing-root refusal must preserve all retained bytes"
    );
    Ok(())
}

// Size: medium. Oracle: Core Law refuses checksum corruption and preserves the exact evidence.
// Delete only when stronger filesystem recovery coverage subsumes this corruption law.
#[test]
fn checksum_corrupt_root_stage_is_refused_without_changing_evidence() -> Result<(), Box<dyn Error>>
{
    use super::filesystem_retention_test_fixture::retention_witness;
    use super::{
        FilesystemRetentionRecoveryError, RetentionFixedStage, RetentionRecoveryRefusal,
        RetentionRootDecodeError,
    };

    let (sandbox, mut authority) = open_authority("recovery-checksum-corrupt-root")?;
    let root_bytes = fixture(ROOT_HEX)?;
    let mut corrupt = root_bytes.clone();
    *corrupt.last_mut().ok_or("empty root fixture")? ^= 0x01;
    let checksum_offset = root_bytes.len().checked_sub(32).ok_or("missing checksum")?;
    let expected_checksum: [u8; 32] = root_bytes
        .get(checksum_offset..)
        .ok_or("missing checksum")?
        .try_into()?;
    let observed_checksum: [u8; 32] = corrupt
        .get(checksum_offset..)
        .ok_or("missing damaged checksum")?
        .try_into()?;
    fs::write(sandbox.path().join("retention/root.next"), &corrupt)?;
    let before = retention_witness(sandbox.path())?;

    let result = authority.recover();

    assert!(
        matches!(&result,
            Err(FilesystemRetentionRecoveryError::Plan {
                source: RetentionRecoveryRefusal::StageCorrupt {
                    stage: RetentionFixedStage::Root, source,
                },
            }) if matches!(source.downcast_ref::<RetentionRootDecodeError>(),
                Some(RetentionRootDecodeError::ChecksumMismatch { expected, observed })
                    if *expected == expected_checksum && *observed == observed_checksum)
        ),
        "corrupt root stage must report its exact checksum refusal: {result:?}"
    );
    assert_eq!(
        retention_witness(sandbox.path())?,
        before,
        "corrupt-stage refusal must preserve all retained bytes"
    );
    Ok(())
}
