//! Medium filesystem laws: final-head failures preserve earlier completion and
//! the published namespace. Oracle: successful rename/unlink is not rolled back
//! by a port-injected EIO. These are not physical power-loss experiments.
//! Delete only when recovery is removed or stronger runtime laws subsume this.

use std::{error::Error, fs, io};

use super::recovery::{CompleteStageEvidence, recover_scheduled};
use super::test_fixture::pool_segment_bytes;
use super::{
    CompactionRecoveryAction as Action, CompactionRecoveryBoundary as Boundary,
    CompactionRecoveryEffects as Effects, recover_compaction,
};
use crate::CatalogGeneration;
use crate::adapters::retention::filesystem_retention_test_fixture::{
    catalog_policy, migrated_store,
};
use crate::adapters::{
    AdmittedSegment, CanonicalCatalog, CanonicalPublicationHead, FilesystemCatalogSnapshot,
    FilesystemRecoveryStageDiscarder, RecoveryStage, physical_pool_name,
};

#[test]
fn root_sync_failure_reports_the_published_head() -> Result<(), Box<dyn Error>> {
    let store = migrated_store("compaction-head-progress")?;
    let segment = pool_segment_bytes(store.path())?
        .pop()
        .ok_or("missing segment")?;
    let head = install_successor(store.path(), &segment)?;
    fs::write(store.path().join("staging/current.seg"), &segment)?;
    fs::write(store.path().join("staging/current.cat"), b"K")?;
    let discarder = FilesystemRecoveryStageDiscarder::open_version_two(store.path())?;

    let error = recover_scheduled(
        discarder,
        catalog_policy()?,
        CompleteStageEvidence::Derivable,
        || Ok(()),
        &mut |_, boundary| {
            if boundary == Boundary::SynchronizeRoot {
                Err(io::Error::from_raw_os_error(5))
            } else {
                Ok(())
            }
        },
    )
    .err()
    .ok_or("failed root sync returned success")?;

    let progress = error
        .execution()
        .ok_or("head failure lost execution progress")?;
    assert_eq!(
        progress.action(),
        Action::Finalize(CatalogGeneration::new(2)?)
    );
    assert_eq!(progress.boundary(), Boundary::SynchronizeRoot);
    assert_eq!(progress.effects(), Effects::AppliedUnconfirmed);
    assert_eq!(
        progress.completed().discarded(),
        &[RecoveryStage::Segment, RecoveryStage::Catalog]
    );
    assert_eq!(
        progress.completed().finalized(),
        None,
        "failed finalization is not a durable completion"
    );
    assert_eq!(
        fs::read(store.path().join("HEAD"))?,
        head,
        "failed sync did not undo rename"
    );
    assert!(
        !store.path().join("head.next").try_exists()?,
        "rename consumed head.next"
    );
    assert!(!store.path().join("staging/current.seg").try_exists()?);
    assert!(!store.path().join("staging/current.cat").try_exists()?);
    assert!(
        recover_compaction(store.path(), catalog_policy()?)?.was_idle(),
        "reopened recovery observes the completed namespace; this is not a sync receipt"
    );
    store.remove()?;
    Ok(())
}

#[test]
fn head_discard_sync_failure_reports_completed_stage_cleanup() -> Result<(), Box<dyn Error>> {
    let store = migrated_store("compaction-head-discard-progress")?;
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
        &mut |stage, boundary| {
            if stage == RecoveryStage::NextHead && boundary == Boundary::SynchronizeParent {
                Err(io::Error::from_raw_os_error(5))
            } else {
                Ok(())
            }
        },
    )
    .err()
    .ok_or("failed head discard sync returned success")?;

    let progress = error
        .execution()
        .ok_or("head discard lost execution progress")?;
    assert_eq!(progress.action(), Action::Discard(RecoveryStage::NextHead));
    assert_eq!(progress.boundary(), Boundary::SynchronizeParent);
    assert_eq!(progress.effects(), Effects::AppliedUnconfirmed);
    assert_eq!(
        progress.completed().discarded(),
        &[RecoveryStage::Segment, RecoveryStage::Catalog]
    );
    assert!(!store.path().join("head.next").try_exists()?);
    assert_eq!(
        fs::read(store.path().join("HEAD"))?,
        before,
        "discard did not publish"
    );
    assert!(recover_compaction(store.path(), catalog_policy()?)?.was_idle());
    store.remove()?;
    Ok(())
}

fn install_successor(root: &std::path::Path, bytes: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    let policy = catalog_policy()?;
    let current = FilesystemCatalogSnapshot::load(root, policy)?;
    let snapshot = current.snapshot()?;
    let segment = AdmittedSegment::decode(bytes, policy.segment_read())?;
    let catalog = CanonicalCatalog::from_segments(
        CatalogGeneration::new(2)?,
        Some(snapshot.catalog_digest()),
        &[segment],
    )?;
    let admitted = catalog.checksummed();
    let name = physical_pool_name::catalog(admitted.generation(), admitted.digest());
    let head = CanonicalPublicationHead::for_catalog(admitted);
    fs::write(root.join("catalogs").join(name), catalog.encoded())?;
    fs::write(root.join("head.next"), head.encoded())?;
    Ok(head.encoded().to_vec())
}
