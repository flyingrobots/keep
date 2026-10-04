//! Medium filesystem laws: observed stage identity survives recovery preflight.
//! Oracle: byte-equal replacement is not the file admitted for cleanup.
//! Delete only if recovery is removed or a stronger schedule test subsumes this law.

use std::{error::Error, fs, os::unix::fs::MetadataExt};

use super::recovery::{CompleteStageEvidence, recover_after_preflight};
use super::test_fixture::{mixed_store, pool_segment_bytes};
use crate::adapters::retention::filesystem_retention_test_fixture::catalog_policy;
use crate::adapters::{
    FilesystemRecoveryStageDiscarder, FilesystemRecoveryStageError, RecoveryStage,
};

#[test]
fn byte_equal_segment_replacement_is_preserved_before_cleanup() -> Result<(), Box<dyn Error>> {
    refuses_substitution(RecoveryStage::Segment)
}

#[test]
fn byte_equal_later_catalog_replacement_preserves_earlier_stage() -> Result<(), Box<dyn Error>> {
    refuses_substitution(RecoveryStage::Catalog)
}

fn refuses_substitution(stage: RecoveryStage) -> Result<(), Box<dyn Error>> {
    let sandbox = mixed_store(&format!("compaction-recovery-identity-{stage:?}"))?;
    let segment = pool_segment_bytes(sandbox.path())?
        .pop()
        .ok_or("missing segment")?;
    let staging = sandbox.path().join("staging");
    fs::write(staging.join("current.seg"), &segment)?;
    fs::write(staging.join("current.cat"), b"K")?;
    let path = staging.join(stage.file_name());
    let original = fs::File::open(&path)?;
    let original_inode = original.metadata()?.ino();
    let bytes = fs::read(&path)?;
    let head = fs::read(sandbox.path().join("HEAD"))?;
    let discarder = FilesystemRecoveryStageDiscarder::open_version_two(sandbox.path())?;

    let result = recover_after_preflight(
        discarder,
        catalog_policy()?,
        CompleteStageEvidence::Derivable,
        || {
            let replacement = staging.join("replacement");
            fs::write(&replacement, &bytes)?;
            fs::rename(replacement, &path)
        },
    );

    let error = result
        .err()
        .ok_or("byte-equal replacement was silently cleaned")?;
    let mut cause: &(dyn Error + 'static) = &error;
    while cause
        .downcast_ref::<FilesystemRecoveryStageError>()
        .is_none()
    {
        cause = cause
            .source()
            .ok_or("stage identity refusal source missing")?;
    }
    assert!(
        matches!(cause.downcast_ref::<FilesystemRecoveryStageError>(),
        Some(FilesystemRecoveryStageError::Replaced { stage: observed }) if *observed == stage)
    );
    assert_ne!(
        fs::metadata(&path)?.ino(),
        original_inode,
        "replacement remains present"
    );
    assert_eq!(
        fs::read(staging.join("current.seg"))?,
        segment,
        "earlier evidence remains"
    );
    assert_eq!(
        fs::read(staging.join("current.cat"))?,
        b"K",
        "later evidence remains"
    );
    assert_eq!(
        fs::read(sandbox.path().join("HEAD"))?,
        head,
        "publication remains unchanged"
    );
    sandbox.remove()?;
    Ok(())
}
