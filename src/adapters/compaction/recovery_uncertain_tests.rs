//! Medium Linux filesystem law: an actual failed unlink is not reported as a
//! completed mutation or an effect-free admission refusal. Oracle: preserve the
//! syscall cause and all evidence; an unconfirmed mutation requires observation.
//! The deliberate raw substitution tests diagnostics, not writer-lock isolation.
//! Delete when recovery is removed or stronger runtime laws subsume this claim.

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
fn failed_unlink_reports_uncertainty_without_losing_evidence() -> Result<(), Box<dyn Error>> {
    let store = migrated_store("compaction-failed-unlink-progress")?;
    let segment = pool_segment_bytes(store.path())?
        .pop()
        .ok_or("missing segment")?;
    let stage = store.path().join("staging/current.seg");
    let retained = store.path().join("retained-test-evidence");
    fs::write(&stage, &segment)?;
    fs::write(store.path().join("staging/current.cat"), b"K")?;
    let head = fs::read(store.path().join("HEAD"))?;
    let discarder = FilesystemRecoveryStageDiscarder::open_version_two(store.path())?;

    let error = recover_scheduled(
        discarder,
        catalog_policy()?,
        CompleteStageEvidence::Derivable,
        || Ok(()),
        &mut |name, boundary| {
            if name == RecoveryStage::Segment && boundary == Boundary::RemoveStage {
                fs::rename(&stage, &retained)?;
                fs::create_dir(&stage)?;
            }
            Ok(())
        },
    )
    .err()
    .ok_or("unlink of a directory unexpectedly succeeded")?;

    let progress = error
        .execution()
        .ok_or("failed unlink lost execution progress")?;
    assert_eq!(progress.action(), Action::Discard(RecoveryStage::Segment));
    assert_eq!(progress.boundary(), Boundary::RemoveStage);
    assert_eq!(progress.effects(), Effects::Uncertain);
    assert!(
        progress.completed().was_idle(),
        "no earlier action completed"
    );
    let cause = error
        .source()
        .and_then(|source| source.downcast_ref::<io::Error>())
        .ok_or("failed unlink lost its I/O cause")?;
    assert_eq!(
        cause.raw_os_error(),
        Some(21),
        "Linux unlink EISDIR remains exact"
    );
    assert!(
        stage.is_dir(),
        "the failed operation preserved the substituted entry"
    );
    assert_eq!(
        fs::read(&retained)?,
        segment,
        "original opened evidence was preserved"
    );
    assert_eq!(
        fs::read(store.path().join("staging/current.cat"))?,
        b"K",
        "later action did not run"
    );
    assert_eq!(fs::read(store.path().join("HEAD"))?, head);
    fs::remove_dir(&stage)?;
    fs::rename(&retained, &stage)?;
    let _receipt = recover_compaction(store.path(), catalog_policy()?)?;
    assert!(recover_compaction(store.path(), catalog_policy()?)?.was_idle());
    store.remove()?;
    Ok(())
}
