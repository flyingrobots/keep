//! Medium filesystem laws for failed recovery actions and their known effects.
//! Oracle: refusal is not rollback; earlier durable actions and failing-action
//! effects remain separate. Deterministic I/O checkpoints inject EIO, not power loss.
//! Delete only when recovery is removed or stronger runtime evidence subsumes this.

use std::{error::Error, fs, io};

use super::recovery::{CompleteStageEvidence, recover_scheduled};
use super::test_fixture::pool_segment_bytes;
use super::{
    CompactionRecoveryAction as Action, CompactionRecoveryBoundary as Boundary,
    CompactionRecoveryEffects as Effects, recover_compaction,
};
use crate::adapters::retention::filesystem_retention_test_fixture::{
    catalog_policy, migrated_store,
};
use crate::adapters::{FilesystemRecoveryStageDiscarder, RecoveryStage};

#[test]
fn failure_before_removal_reports_no_namespace_effect() -> Result<(), Box<dyn Error>> {
    check_discard_failure(RecoveryStage::Segment, Boundary::RemoveStage, Effects::None)
}

#[test]
fn failed_absence_check_reports_the_unlinked_stage() -> Result<(), Box<dyn Error>> {
    check_discard_failure(
        RecoveryStage::Segment,
        Boundary::ConfirmStageAbsent,
        Effects::AppliedUnconfirmed,
    )
}

#[test]
fn failed_parent_sync_reports_the_unlinked_stage() -> Result<(), Box<dyn Error>> {
    check_discard_failure(
        RecoveryStage::Segment,
        Boundary::SynchronizeParent,
        Effects::AppliedUnconfirmed,
    )
}

#[test]
fn later_discard_failure_retains_the_earlier_durable_completion() -> Result<(), Box<dyn Error>> {
    check_discard_failure(
        RecoveryStage::Catalog,
        Boundary::SynchronizeParent,
        Effects::AppliedUnconfirmed,
    )
}

fn check_discard_failure(
    failed_stage: RecoveryStage,
    boundary: Boundary,
    effects: Effects,
) -> Result<(), Box<dyn Error>> {
    let store = migrated_store(&format!(
        "compaction-progress-{failed_stage:?}-{boundary:?}"
    ))?;
    let segment = pool_segment_bytes(store.path())?
        .pop()
        .ok_or("missing segment")?;
    fs::write(store.path().join("staging/current.seg"), &segment)?;
    fs::write(store.path().join("staging/current.cat"), b"K")?;
    fs::write(store.path().join("head.next"), b"K")?;
    let before = fs::read(store.path().join("HEAD"))?;
    let discarder = FilesystemRecoveryStageDiscarder::open_version_two(store.path())?;

    let error = recover_scheduled(
        discarder,
        catalog_policy()?,
        CompleteStageEvidence::Derivable,
        || Ok(()),
        &mut |stage, point| {
            if stage == failed_stage && point == boundary {
                Err(io::Error::from_raw_os_error(5))
            } else {
                Ok(())
            }
        },
    )
    .err()
    .ok_or("injected failure returned success")?;

    let progress = error
        .execution()
        .ok_or("recovery lost execution progress")?;
    assert_eq!(progress.action(), Action::Discard(failed_stage));
    assert_eq!(progress.boundary(), boundary);
    assert_eq!(progress.effects(), effects);
    let completed: &[RecoveryStage] = match failed_stage {
        RecoveryStage::Catalog => &[RecoveryStage::Segment],
        _ => &[],
    };
    assert_eq!(
        progress.completed().discarded(),
        completed,
        "earlier durable actions remain reported"
    );
    assert_eq!(progress.completed().finalized(), None);
    assert_eq!(
        fs::read(store.path().join("HEAD"))?,
        before,
        "failed cleanup never publishes"
    );
    assert_eq!(
        fs::read(store.path().join("head.next"))?,
        b"K",
        "no later recovery action runs"
    );
    assert_eq!(
        store.path().join("staging/current.seg").try_exists()?,
        effects == Effects::None,
        "the reported segment effect agrees with the filesystem"
    );
    assert_eq!(
        store.path().join("staging/current.cat").try_exists()?,
        failed_stage != RecoveryStage::Catalog
    );
    let _receipt = recover_compaction(store.path(), catalog_policy()?)?;
    assert!(
        recover_compaction(store.path(), catalog_policy()?)?.was_idle(),
        "fresh recovery finishes interrupted work"
    );
    store.remove()?;
    Ok(())
}
