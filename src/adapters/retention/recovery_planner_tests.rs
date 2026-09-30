//! Retention recovery planning laws over the golden version-two records.

use std::error::Error;

use super::filesystem_retention_test_fixture::{
    HEAD_HEX, MANIFEST_HEX, ROOT_HEX, fixture, initial_preparation, successor_preparation,
};
use super::{
    AdmittedRetentionManifest, AdmittedRetentionRoot, ObservedRetentionState, RetentionFixedStage,
    RetentionPool, RetentionPoolEntryObservation as Pool, RetentionPoolObservations,
    RetentionRecoveryEvidence, RetentionRecoveryOutcome as Outcome, RetentionRecoveryRefusal,
    RetentionRecoveryStep as Step, RetentionStageAssessments, assess_head_stage,
    assess_manifest_stage, assess_root_stage, plan_retention_recovery,
};

struct Records {
    root: Vec<u8>,
    manifest: Vec<u8>,
    head: Vec<u8>,
}

fn generation_one() -> Result<Records, Box<dyn Error>> {
    Ok(Records {
        root: fixture(ROOT_HEX)?,
        manifest: fixture(MANIFEST_HEX)?,
        head: fixture(HEAD_HEX)?,
    })
}

type StageBytes<'b> = (Option<&'b [u8]>, Option<&'b [u8]>, Option<&'b [u8]>);

fn evidence<'b, 's>(
    current: Option<&'s ObservedRetentionState>,
    (root, manifest, head): StageBytes<'b>,
    (root_pool, manifest_pool): (Pool, Pool),
) -> RetentionRecoveryEvidence<'b, 's> {
    RetentionRecoveryEvidence::new(
        current,
        RetentionStageAssessments {
            root: assess_root_stage(root),
            manifest: assess_manifest_stage(manifest),
            head: assess_head_stage(head),
        },
        RetentionPoolObservations {
            root: root_pool,
            manifest: manifest_pool,
        },
    )
}

fn corrupt(bytes: &[u8]) -> Vec<u8> {
    let mut bytes = bytes.to_vec();
    if let Some(last) = bytes.last_mut() {
        *last ^= 0x01;
    }
    bytes
}

#[test]
fn a_clean_store_needs_nothing() -> Result<(), Box<dyn Error>> {
    let plan = plan_retention_recovery(evidence(
        None,
        (None, None, None),
        (Pool::Absent, Pool::Absent),
    ))?;
    assert!(plan.steps().is_empty());
    assert_eq!(plan.outcome(), Outcome::Clean);
    Ok(())
}

#[test]
fn a_truncated_root_stage_with_no_later_effect_is_discarded() -> Result<(), Box<dyn Error>> {
    let records = generation_one()?;
    let partial = records
        .root
        .get(..100)
        .ok_or("root fixture shorter than 100 bytes")?;
    let plan = plan_retention_recovery(evidence(
        None,
        (Some(partial), None, None),
        (Pool::Absent, Pool::Absent),
    ))?;
    assert_eq!(plan.steps(), [Step::DiscardRootStage]);
    assert_eq!(plan.outcome(), Outcome::Clean);
    Ok(())
}

#[test]
fn a_truncated_stage_with_a_later_effect_refuses() -> Result<(), Box<dyn Error>> {
    let records = generation_one()?;
    let partial = records
        .manifest
        .get(..100)
        .ok_or("manifest fixture shorter than 100 bytes")?;
    let error = plan_retention_recovery(evidence(
        None,
        (Some(&records.root), Some(partial), None),
        (Pool::Identical, Pool::Identical),
    ))
    .err()
    .ok_or("a truncated manifest with a pool link was discarded")?;
    assert!(matches!(
        error,
        RetentionRecoveryRefusal::TruncatedStageWithLaterEffect {
            stage: RetentionFixedStage::Manifest
        }
    ));
    Ok(())
}

#[test]
fn a_corrupt_stage_refuses_with_its_decode_error() -> Result<(), Box<dyn Error>> {
    let records = generation_one()?;
    let corrupt_root = corrupt(&records.root);
    let error = plan_retention_recovery(evidence(
        None,
        (Some(&corrupt_root), None, None),
        (Pool::Absent, Pool::Absent),
    ))
    .err()
    .ok_or("a corrupt root stage was planned")?;
    assert!(matches!(
        error,
        RetentionRecoveryRefusal::StageCorrupt {
            stage: RetentionFixedStage::Root,
            ..
        }
    ));
    assert!(error.source().is_some());
    Ok(())
}

#[test]
fn a_complete_root_stage_is_linked_and_protected() -> Result<(), Box<dyn Error>> {
    let records = generation_one()?;
    let unlinked = plan_retention_recovery(evidence(
        None,
        (Some(&records.root), None, None),
        (Pool::Absent, Pool::Absent),
    ))?;
    let linked = plan_retention_recovery(evidence(
        None,
        (Some(&records.root), None, None),
        (Pool::Identical, Pool::Absent),
    ))?;
    let differs = plan_retention_recovery(evidence(
        None,
        (Some(&records.root), None, None),
        (Pool::Different, Pool::Absent),
    ))
    .err()
    .ok_or("a conflicting root pool entry was planned over")?;
    assert_eq!(unlinked.steps(), [Step::LinkRoot]);
    assert!(linked.steps().is_empty());
    for plan in [&unlinked, &linked] {
        assert_eq!(
            plan.outcome(),
            Outcome::Protected {
                root_stage: true,
                manifest_stage: false
            }
        );
    }
    assert!(matches!(
        differs,
        RetentionRecoveryRefusal::PoolEntryDiffers {
            pool: RetentionPool::Roots
        }
    ));
    Ok(())
}

#[test]
fn complete_root_and_manifest_stages_are_linked_and_protected() -> Result<(), Box<dyn Error>> {
    let records = generation_one()?;
    let plan = plan_retention_recovery(evidence(
        None,
        (Some(&records.root), Some(&records.manifest), None),
        (Pool::Absent, Pool::Absent),
    ))?;
    assert_eq!(plan.steps(), [Step::LinkRoot, Step::LinkManifest]);
    assert_eq!(
        plan.outcome(),
        Outcome::Protected {
            root_stage: true,
            manifest_stage: true
        }
    );
    Ok(())
}

#[test]
fn a_complete_head_stage_over_linked_stages_is_finalized() -> Result<(), Box<dyn Error>> {
    let records = generation_one()?;
    let plan = plan_retention_recovery(evidence(
        None,
        (
            Some(&records.root),
            Some(&records.manifest),
            Some(&records.head),
        ),
        (Pool::Identical, Pool::Identical),
    ))?;
    assert_eq!(
        plan.steps(),
        [
            Step::FinalizeHead,
            Step::RemoveRootStage,
            Step::RemoveManifestStage
        ]
    );
    assert_eq!(plan.outcome(), Outcome::Committed);
    let unlinked = plan_retention_recovery(evidence(
        None,
        (
            Some(&records.root),
            Some(&records.manifest),
            Some(&records.head),
        ),
        (Pool::Absent, Pool::Identical),
    ))
    .err()
    .ok_or("a head stage over an unlinked root was finalized")?;
    assert!(matches!(
        unlinked,
        RetentionRecoveryRefusal::RootNotLinkedBeforeHead
    ));
    Ok(())
}

#[test]
fn a_truncated_head_stage_is_discarded_and_the_orphans_stay_protected() -> Result<(), Box<dyn Error>>
{
    let records = generation_one()?;
    let partial = records
        .head
        .get(..40)
        .ok_or("head fixture shorter than 40 bytes")?;
    let plan = plan_retention_recovery(evidence(
        None,
        (Some(&records.root), Some(&records.manifest), Some(partial)),
        (Pool::Identical, Pool::Identical),
    ))?;
    assert_eq!(plan.steps(), [Step::DiscardHeadStage]);
    assert_eq!(
        plan.outcome(),
        Outcome::Protected {
            root_stage: true,
            manifest_stage: true
        }
    );
    Ok(())
}

#[test]
fn a_head_stage_without_the_staged_manifest_refuses() -> Result<(), Box<dyn Error>> {
    let records = generation_one()?;
    let error = plan_retention_recovery(evidence(
        None,
        (Some(&records.root), None, Some(&records.head)),
        (Pool::Identical, Pool::Absent),
    ))
    .err()
    .ok_or("a head stage without a manifest stage was finalized")?;
    assert!(matches!(
        error,
        RetentionRecoveryRefusal::HeadStageWithoutManifestStage
    ));
    Ok(())
}

#[test]
fn stages_the_published_head_already_names_are_cleaned_up() -> Result<(), Box<dyn Error>> {
    let records = generation_one()?;
    let current = ObservedRetentionState::for_tests(&records.head, &records.manifest)?;
    let both = plan_retention_recovery(evidence(
        Some(&current),
        (Some(&records.root), Some(&records.manifest), None),
        (Pool::Identical, Pool::Identical),
    ))?;
    let manifest_only = plan_retention_recovery(evidence(
        Some(&current),
        (None, Some(&records.manifest), None),
        (Pool::Absent, Pool::Identical),
    ))?;
    let head_too = plan_retention_recovery(evidence(
        Some(&current),
        (
            Some(&records.root),
            Some(&records.manifest),
            Some(&records.head),
        ),
        (Pool::Identical, Pool::Identical),
    ))?;
    assert_eq!(
        both.steps(),
        [Step::RemoveRootStage, Step::RemoveManifestStage]
    );
    assert_eq!(manifest_only.steps(), [Step::RemoveManifestStage]);
    assert_eq!(
        head_too.steps(),
        [
            Step::FinalizeHead,
            Step::RemoveRootStage,
            Step::RemoveManifestStage
        ]
    );
    for plan in [&both, &manifest_only, &head_too] {
        assert_eq!(plan.outcome(), Outcome::Committed);
    }
    Ok(())
}

#[test]
fn a_successor_generation_is_planned_against_the_published_state() -> Result<(), Box<dyn Error>> {
    let records = generation_one()?;
    let current = ObservedRetentionState::for_tests(&records.head, &records.manifest)?;
    let current_root = AdmittedRetentionRoot::decode(&records.root)?;
    let current_manifest = AdmittedRetentionManifest::decode(&records.manifest)?;
    let candidate = super::filesystem_retention_test_fixture::successor_root(&current_root)?;
    let preparation = successor_preparation(&current_root, &current_manifest, candidate.encoded())?;
    let publication = preparation
        .publication()
        .ok_or("successor preparation carries no publication")?;
    let manifest = publication.manifest().encoded().to_vec();
    let head = publication.head().encoded().to_vec();

    let finalize = plan_retention_recovery(evidence(
        Some(&current),
        (Some(candidate.encoded()), Some(&manifest), Some(&head)),
        (Pool::Identical, Pool::Identical),
    ))?;
    let stale = plan_retention_recovery(evidence(
        None,
        (Some(candidate.encoded()), Some(&manifest), None),
        (Pool::Absent, Pool::Absent),
    ))
    .err()
    .ok_or("a successor manifest over an absent head was planned")?;

    assert_eq!(
        finalize.steps(),
        [
            Step::FinalizeHead,
            Step::RemoveRootStage,
            Step::RemoveManifestStage
        ]
    );
    assert_eq!(finalize.outcome(), Outcome::Committed);
    assert!(matches!(
        stale,
        RetentionRecoveryRefusal::ManifestNotSuccessor
    ));
    drop(initial_preparation(&records.root)?);
    Ok(())
}
