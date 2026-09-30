//! Filesystem GC liveness laws over the migrated fixture store.

use std::error::Error;
use std::fs;
use std::path::Path;

use super::{
    GcLimits, GcLivenessObservationError, GcRetentionState, GcSegmentClassification,
    observe_gc_liveness, plan_gc,
};
use crate::adapters::retention::filesystem_retention_test_fixture::{
    ROOT_HEX, catalog_policy, fixture, initial_preparation, migrated_store, reopen_authority,
};
use crate::adapters::test_support::decode_hex;
use crate::adapters::{FilesystemRetentionSnapshot, ReaderAttemptLimit};
use crate::execute_retention_publication;

const ORPHAN_SEGMENT_HEX: &str =
    include_str!("../../../conformance/segment-store/v1/one-zero-segment.hex");
const ORPHAN_SEGMENT_NAME: &str =
    "b7542dced2ab770894a14d1d04b066e3a899942602c5986d35ba6df6c1a35cfc.seg";

fn published_store(
    name: &str,
) -> Result<crate::adapters::filesystem_test_sandbox::TestDirectory, Box<dyn Error>> {
    let sandbox = migrated_store(name)?;
    let root_bytes = fixture(ROOT_HEX)?;
    let preparation = initial_preparation(&root_bytes)?;
    let mut authority = reopen_authority(sandbox.path())?;
    let _published = execute_retention_publication(&mut authority, &preparation)?;
    drop(authority);
    Ok(sandbox)
}

fn observe(root: &Path) -> Result<super::GcLivenessSnapshot, Box<dyn Error>> {
    let view =
        FilesystemRetentionSnapshot::load(root, catalog_policy()?, ReaderAttemptLimit::DEFAULT)?;
    observe_gc_liveness(root, &view, catalog_policy()?).map_err(Into::into)
}

#[test]
fn the_published_fixture_store_plans_its_one_segment_live() -> Result<(), Box<dyn Error>> {
    let sandbox = published_store("gc-liveness-published")?;

    let snapshot = observe(sandbox.path())?;
    let plan = plan_gc(&snapshot, GcLimits::MAXIMUM)?;

    assert!(matches!(
        snapshot.coordinates().retention(),
        GcRetentionState::Published { .. }
    ));
    assert_eq!(snapshot.retained().len(), 1);
    assert_eq!(plan.segments().len(), 1);
    assert!(plan.segments().values().all(|planned| {
        planned.classification() == GcSegmentClassification::Live { retained_roots: 1 }
    }));
    assert_eq!(plan.candidate_count(), 0);
    sandbox.remove()?;
    Ok(())
}

#[test]
fn an_unpublished_store_plans_its_named_segment_unreachable_but_not_collectible()
-> Result<(), Box<dyn Error>> {
    let sandbox = migrated_store("gc-liveness-empty-retention")?;

    let snapshot = observe(sandbox.path())?;
    let plan = plan_gc(&snapshot, GcLimits::MAXIMUM)?;

    assert_eq!(snapshot.coordinates().retention(), GcRetentionState::Empty);
    assert!(snapshot.retained().is_empty());
    assert!(
        plan.segments().values().all(|planned| {
            planned.classification() == GcSegmentClassification::NamedUnreachable
        })
    );
    assert_eq!(plan.candidate_count(), 0);
    sandbox.remove()?;
    Ok(())
}

#[test]
fn an_orphan_pool_segment_is_recovery_protected_and_never_a_candidate() -> Result<(), Box<dyn Error>>
{
    let sandbox = published_store("gc-liveness-orphan")?;
    let orphan = decode_hex(ORPHAN_SEGMENT_HEX.trim())?;
    fs::write(
        sandbox.path().join("segments").join(ORPHAN_SEGMENT_NAME),
        &orphan,
    )?;

    let snapshot = observe(sandbox.path())?;
    let plan = plan_gc(&snapshot, GcLimits::MAXIMUM)?;

    assert_eq!(plan.segments().len(), 2);
    let protected = plan
        .segments()
        .values()
        .filter(|planned| planned.classification() == GcSegmentClassification::RecoveryProtected)
        .count();
    assert_eq!(protected, 1);
    assert_eq!(plan.candidate_count(), 0);
    sandbox.remove()?;
    Ok(())
}

#[test]
fn a_corrupt_pool_segment_refuses_observation_before_any_plan() -> Result<(), Box<dyn Error>> {
    let sandbox = published_store("gc-liveness-corrupt")?;
    let mut orphan = decode_hex(ORPHAN_SEGMENT_HEX.trim())?;
    let last = orphan.last_mut().ok_or("orphan is empty")?;
    *last ^= 1;
    fs::write(
        sandbox.path().join("segments").join(ORPHAN_SEGMENT_NAME),
        &orphan,
    )?;

    let error = observe(sandbox.path())
        .err()
        .ok_or("a corrupt pool segment was unexpectedly observed")?;

    let error = error
        .downcast::<GcLivenessObservationError>()
        .map_err(|_error| "observation refused with the wrong error type")?;
    assert!(matches!(
        *error,
        GcLivenessObservationError::SegmentAdmission { .. }
    ));
    sandbox.remove()?;
    Ok(())
}

#[test]
fn a_pool_entry_not_named_by_a_digest_refuses_observation() -> Result<(), Box<dyn Error>> {
    let sandbox = published_store("gc-liveness-stray")?;
    fs::write(sandbox.path().join("segments").join("stray.seg"), b"x")?;

    let error = observe(sandbox.path())
        .err()
        .ok_or("a stray pool entry was unexpectedly observed")?;

    let error = error
        .downcast::<GcLivenessObservationError>()
        .map_err(|_error| "observation refused with the wrong error type")?;
    assert!(matches!(*error, GcLivenessObservationError::PoolEntryName));
    sandbox.remove()?;
    Ok(())
}
