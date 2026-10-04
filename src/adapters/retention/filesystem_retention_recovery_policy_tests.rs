//! These laws own explicit recovery catalog-memory admission at its boundary.

use std::error::Error;
use std::fs;

use super::filesystem_retention_test_fixture::{
    ROOT_HEX, SEGMENT_NAME, drive_publication, fixture, initial_preparation, open_authority,
    retention_witness,
};
use super::{FilesystemRetentionRecoveryError, RetentionRecoveryOutcome};
use crate::adapters::{
    CatalogRestartByteLimit, CatalogRestartError, CatalogRestartPolicy, SegmentReadPolicy,
};

// Size: medium. Oracle: aggregate segment retention refuses before exceeding caller budget.
// Delete only with explicit recovery policy or a stronger runtime replacement.
#[test]
fn recovery_refuses_a_catalog_budget_smaller_than_selected_segments() -> Result<(), Box<dyn Error>>
{
    let (sandbox, mut authority) = open_authority("recovery-catalog-budget-refusal")?;
    let bytes = fixture(ROOT_HEX)?;
    drive_publication(&mut authority, &initial_preparation(&bytes)?, 2)?;
    let observed = fs::metadata(sandbox.path().join("segments").join(SEGMENT_NAME))?.len();
    let maximum = observed.checked_sub(1).ok_or("empty selected segment")?;
    let policy = CatalogRestartPolicy::new(
        SegmentReadPolicy::MAXIMUM,
        CatalogRestartByteLimit::new(maximum)?,
    );
    let before = retention_witness(sandbox.path())?;
    let result = authority.recover_with_catalog_policy(policy);
    assert_eq!(
        retention_witness(sandbox.path())?,
        before,
        "catalog budget refusal must preserve retained evidence: {result:?}"
    );
    let error = match &result {
        Err(FilesystemRetentionRecoveryError::Observe { source }) => source,
        other => {
            return Err(format!("catalog budget must refuse during observation: {other:?}").into());
        }
    };
    let restart = error
        .get_ref()
        .and_then(|source| source.downcast_ref::<CatalogRestartError>());
    assert!(
        matches!(restart, Some(CatalogRestartError::RetainedSegmentBytes {
        maximum: actual_maximum, observed: actual_observed
    }) if *actual_maximum == maximum && *actual_observed == observed),
        "catalog budget must name exact maximum {maximum} and observed {observed}: {result:?}"
    );
    Ok(())
}

// Size: medium. Oracle: an inclusive aggregate byte budget admits exact-size selection.
// Delete only with explicit recovery policy or a stronger runtime replacement.
#[test]
fn recovery_admits_a_catalog_budget_equal_to_selected_segment_bytes() -> Result<(), Box<dyn Error>>
{
    let (sandbox, mut authority) = open_authority("recovery-catalog-budget-equality")?;
    let bytes = fixture(ROOT_HEX)?;
    drive_publication(&mut authority, &initial_preparation(&bytes)?, 2)?;
    let maximum = fs::metadata(sandbox.path().join("segments").join(SEGMENT_NAME))?.len();
    let policy = CatalogRestartPolicy::new(
        SegmentReadPolicy::MAXIMUM,
        CatalogRestartByteLimit::new(maximum)?,
    );
    let result = authority.recover_with_catalog_policy(policy);
    assert!(
        matches!(result, Ok(ref receipt) if receipt.outcome() == RetentionRecoveryOutcome::Protected {
            root_stage: true, manifest_stage: false
        }),
        "exact-size catalog budget must protect the admitted root: {result:?}"
    );
    Ok(())
}
