//! Truncation-only discard-planning laws.

use std::error::Error;

use keep::{
    RecoverySegmentTruncation, RecoveryStage, RecoveryStageDiscardPlanError,
    RecoveryStageDiscardReason, plan_recovery_stage_discard,
};

use super::{
    CATALOG_HEX, HEAD_HEX, SEGMENT_HEX, SEGMENT_SEAL_LENGTH, assessment, evidence, fixture,
};

#[test]
fn every_exact_truncation_retains_its_reason_and_evidence() -> Result<(), Box<dyn Error>> {
    let segment = [0_u8];
    let catalog = [0_u8];
    let head = [0_u8];
    let segment_evidence = evidence(RecoveryStage::Segment, &segment)?;
    let catalog_evidence = evidence(RecoveryStage::Catalog, &catalog)?;
    let head_evidence = evidence(RecoveryStage::NextHead, &head)?;

    let segment_request =
        plan_recovery_stage_discard(&assessment(RecoveryStage::Segment, &segment)?)?;
    let catalog_request =
        plan_recovery_stage_discard(&assessment(RecoveryStage::Catalog, &catalog)?)?;
    let head_request = plan_recovery_stage_discard(&assessment(RecoveryStage::NextHead, &head)?)?;

    assert_eq!(segment_request.evidence(), segment_evidence);
    assert_eq!(catalog_request.evidence(), catalog_evidence);
    assert_eq!(head_request.evidence(), head_evidence);
    assert!(matches!(
        segment_request.reason(),
        RecoveryStageDiscardReason::Segment(RecoverySegmentTruncation::Header { observed: 1, .. })
    ));
    assert!(matches!(
        catalog_request.reason(),
        RecoveryStageDiscardReason::CatalogHeader { observed: 1, .. }
    ));
    assert!(matches!(
        head_request.reason(),
        RecoveryStageDiscardReason::NextHead { observed: 1, .. }
    ));
    Ok(())
}

#[test]
fn complete_stages_cannot_form_truncation_discard_requests() -> Result<(), Box<dyn Error>> {
    for (stage, encoded) in [
        (RecoveryStage::Segment, fixture(SEGMENT_HEX)?),
        (RecoveryStage::Catalog, fixture(CATALOG_HEX)?),
        (RecoveryStage::NextHead, fixture(HEAD_HEX)?),
    ] {
        let assessed = assessment(stage, &encoded)?;
        let error = plan_recovery_stage_discard(&assessed)
            .err()
            .ok_or("complete stage formed a truncation-discard request")?;

        assert!(matches!(
            error,
            RecoveryStageDiscardPlanError::NotTruncated { stage: observed }
                if observed == stage
        ));
    }
    Ok(())
}

#[test]
fn reusable_segment_prefix_cannot_form_a_discard_request() -> Result<(), Box<dyn Error>> {
    let complete = fixture(SEGMENT_HEX)?;
    let prefix_length = complete
        .len()
        .checked_sub(SEGMENT_SEAL_LENGTH)
        .ok_or("canonical segment is shorter than its fixed seal")?;
    let prefix = complete
        .get(..prefix_length)
        .ok_or("canonical reusable prefix is unavailable")?;
    let assessed = assessment(RecoveryStage::Segment, prefix)?;

    let error = plan_recovery_stage_discard(&assessed)
        .err()
        .ok_or("reusable segment prefix formed a discard request")?;

    assert_eq!(
        error,
        RecoveryStageDiscardPlanError::NotTruncated {
            stage: RecoveryStage::Segment,
        }
    );
    Ok(())
}
