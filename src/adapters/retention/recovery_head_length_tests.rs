//! These laws own the planner's exact head-to-manifest length contract.

use std::error::Error;

use super::filesystem_retention_test_fixture::{MANIFEST_HEX, ROOT_HEX, fixture};
use super::{
    AdmittedRetentionManifest, CanonicalRetentionHead, RetentionPoolEntryObservation as Pool,
    RetentionPoolObservations, RetentionRecoveryEvidence, RetentionRecoveryOutcome,
    RetentionRecoveryRefusal, RetentionStageAssessments, assess_head_stage, assess_manifest_stage,
    assess_root_stage, plan_retention_recovery,
};
use crate::{RetentionHead, RetentionManifest, RetentionManifestLength};

// Size: small. Oracle: every admitted head length must equal the manifest bytes.
// Exhaustive finite input sweep; the first failing length is a minimal reproducer.
// Delete only when a stronger public planner property subsumes this contract.
#[test]
fn recovery_binds_head_length_across_the_complete_canonical_length_domain()
-> Result<(), Box<dyn Error>> {
    let root = fixture(ROOT_HEX)?;
    let manifest_bytes = fixture(MANIFEST_HEX)?;
    let manifest = AdmittedRetentionManifest::decode(&manifest_bytes)?;
    let actual = u64::try_from(manifest_bytes.len())?;
    for entries in 0..=RetentionManifest::MAXIMUM_ENTRY_COUNT {
        let length = u64::from(entries)
            .checked_mul(72)
            .and_then(|bytes| RetentionManifestLength::MINIMUM.get().checked_add(bytes))
            .ok_or("canonical length sweep overflowed")?;
        let head = RetentionHead::new(
            manifest.manifest().generation(),
            RetentionManifestLength::new(length)?,
            manifest.digest(),
            manifest.manifest().predecessor(),
        )?;
        let encoded = CanonicalRetentionHead::from_head(&head);
        let result = plan_retention_recovery(RetentionRecoveryEvidence::new(
            None,
            RetentionStageAssessments {
                root: assess_root_stage(Some(&root)),
                manifest: assess_manifest_stage(Some(&manifest_bytes)),
                head: assess_head_stage(Some(encoded.encoded())),
            },
            RetentionPoolObservations {
                root: Pool::Identical,
                manifest: Pool::Identical,
            },
        ));
        if length == actual {
            let plan = result?;
            assert_eq!(
                plan.outcome(),
                RetentionRecoveryOutcome::Committed,
                "exact manifest length {length} must remain recoverable"
            );
        } else {
            assert!(
                matches!(
                    result,
                    Err(RetentionRecoveryRefusal::HeadStageNamesOtherManifest)
                ),
                "head length {length} must refuse manifest length {actual}: {result:?}"
            );
        }
    }
    Ok(())
}
