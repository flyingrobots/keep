//! Filesystem compaction laws over a migrated store holding one mixed
//! segment: the plan names exactly the live records to copy, the successor
//! preserves every identity and closure, GC then retires the old segment,
//! refusals happen before any stage, and recovery over an untouched store
//! is idle.

use std::error::Error;

use super::test_fixture::{
    authority, gc_plan, head_generation, logical_view, mixed_store, observe, plan,
    pool_segment_bytes, recovery_is_idle,
};
use super::{
    CompactionRefusal, CompactionSegmentDisposition, FilesystemCompactionError, plan_compaction,
};
use crate::adapters::gc::{FilesystemGcAuthority, GcSegmentClassification, GcUnreachableEvidence};
use crate::adapters::retention::filesystem_retention_test_fixture::{
    ROOT_HEX, catalog_policy, fixture, initial_preparation, migrated_store, reopen_authority,
};
use crate::adapters::{FilesystemVersionTwoAdmission, SegmentDigest};
use crate::execute_retention_publication;

#[test]
fn a_mixed_segment_plans_its_live_records_for_copy_and_the_rest_for_omission()
-> Result<(), Box<dyn Error>> {
    let sandbox = mixed_store("compaction-plan")?;
    let plan = plan(sandbox.path())?;

    assert_eq!(plan.successor_generation().get(), 3);
    assert_eq!(plan.segments().len(), 2);
    assert_eq!(plan.retained().count(), 1);
    assert_eq!(plan.superseded().count(), 1);
    let compacted: Vec<_> = plan
        .segments()
        .values()
        .filter_map(|disposition| match disposition {
            CompactionSegmentDisposition::Compacted { live, unreachable } => {
                Some((live.len(), *unreachable))
            }
            _ => None,
        })
        .collect();
    assert_eq!(compacted, [(2, 1)]);
    assert_eq!(plan.copied_records(), 2);
    assert!(plan.copied_bytes() > 288);
    assert!(plan.reclaimable_bytes() > plan.copied_bytes());
    assert_eq!(
        plan,
        plan_compaction(&observe(sandbox.path())?)?,
        "planning is pure"
    );
    sandbox.remove()?;
    Ok(())
}

#[test]
fn compaction_preserves_every_identity_and_closure_and_frees_the_mixed_segment()
-> Result<(), Box<dyn Error>> {
    let sandbox = mixed_store("compaction-execute")?;
    let plan = plan(sandbox.path())?;
    let before = logical_view(sandbox.path())?;
    let superseded: Vec<SegmentDigest> = plan.superseded().collect();

    let receipt = authority(sandbox.path())?.execute(&plan)?;

    assert_eq!(receipt.generation().get(), 3);
    assert_eq!(head_generation(sandbox.path())?, 3);
    assert!(receipt.new_segment().is_some());
    assert_eq!(
        receipt.superseded().into_iter().collect::<Vec<_>>(),
        superseded
    );
    let after = logical_view(sandbox.path())?;
    assert_eq!(
        before.0, after.0,
        "every retained closure's root and members are unchanged"
    );
    assert_eq!(
        before.1, after.1,
        "every live record is byte-identical at its new location"
    );
    let observation = observe(sandbox.path())?;
    assert_eq!(
        observation.named().len(),
        4,
        "the unreachable chunk is no longer named"
    );
    assert!(matches!(
        plan_compaction(&observation),
        Err(CompactionRefusal::NothingToCompact)
    ));
    let gc = gc_plan(sandbox.path())?;
    assert_eq!(gc.candidate_count(), 1);
    for digest in &superseded {
        assert_eq!(
            gc.classification(*digest),
            Some(GcSegmentClassification::Unreachable(
                GcUnreachableEvidence::Superseded
            ))
        );
    }
    let admission = FilesystemVersionTwoAdmission::reopen_unchecked_for_tests(sandbox.path())?;
    let mut collector = FilesystemGcAuthority::open(admission, sandbox.path(), catalog_policy()?)?;
    let _retired = collector.execute(&gc)?;
    drop(collector);
    assert_eq!(pool_segment_bytes(sandbox.path())?.len(), 2);
    assert_eq!(
        logical_view(sandbox.path())?,
        after,
        "retirement changes no identity"
    );
    sandbox.remove()?;
    Ok(())
}

#[test]
fn refusals_happen_before_any_stage_is_written() -> Result<(), Box<dyn Error>> {
    let unpublished = migrated_store("compaction-refuse-retention")?;
    assert!(matches!(
        plan_compaction(&observe(unpublished.path())?),
        Err(CompactionRefusal::RetentionNotPublished)
    ));
    unpublished.remove()?;

    let all_live = migrated_store("compaction-refuse-nothing")?;
    let root_bytes = fixture(ROOT_HEX)?;
    let preparation = initial_preparation(&root_bytes)?;
    let mut retention = reopen_authority(all_live.path())?;
    let _published = execute_retention_publication(&mut retention, &preparation)?;
    drop(retention);
    assert!(matches!(
        plan_compaction(&observe(all_live.path())?),
        Err(CompactionRefusal::NothingToCompact)
    ));
    all_live.remove()?;

    // A plan executed once cannot execute again: the store re-plans differently.
    let sandbox = mixed_store("compaction-refuse-stale")?;
    let plan = plan(sandbox.path())?;
    let _receipt = authority(sandbox.path())?.execute(&plan)?;
    let error = authority(sandbox.path())?
        .execute(&plan)
        .err()
        .ok_or("a stale plan executed twice")?;
    assert!(
        matches!(
            error,
            FilesystemCompactionError::Refused(CompactionRefusal::NothingToCompact)
        ),
        "{error}"
    );
    assert!(!sandbox.path().join("staging").join("current.seg").exists());
    sandbox.remove()?;
    Ok(())
}

#[test]
fn recovery_over_an_untouched_store_is_idle() -> Result<(), Box<dyn Error>> {
    let sandbox = mixed_store("compaction-recover-idle")?;
    assert!(recovery_is_idle(sandbox.path())?);
    assert_eq!(head_generation(sandbox.path())?, 2);
    sandbox.remove()?;
    Ok(())
}
