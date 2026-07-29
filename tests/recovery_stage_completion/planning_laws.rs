//! Complete-stage recovery planning laws.

use std::error::Error;

use keep::{
    AdmittedSegment, ChecksummedCatalog, RecoveryStage, RecoveryStageCompletionPlanError,
    RecoveryStageCompletionTarget, plan_recovery_stage_completion,
};

use super::{
    CATALOG_HEX, HEAD_HEX, SEGMENT_HEX, SEGMENT_SEAL_LENGTH, assessment, fixture, maximum_policy,
};

#[test]
fn complete_segment_plan_retains_exact_evidence_and_pool_coordinate() -> Result<(), Box<dyn Error>>
{
    let bytes = fixture(SEGMENT_HEX)?;
    let assessed = assessment(RecoveryStage::Segment, &bytes)?;
    let expected = AdmittedSegment::decode(&bytes, maximum_policy())?;

    let request = plan_recovery_stage_completion(&assessed)?;

    assert_eq!(request.evidence(), assessed.evidence());
    assert_eq!(
        request.target(),
        RecoveryStageCompletionTarget::Segment {
            digest: expected.digest(),
        }
    );
    Ok(())
}

#[test]
fn complete_catalog_plan_retains_exact_evidence_and_pool_coordinate() -> Result<(), Box<dyn Error>>
{
    let bytes = fixture(CATALOG_HEX)?;
    let assessed = assessment(RecoveryStage::Catalog, &bytes)?;
    let expected = ChecksummedCatalog::decode(&bytes)?;

    let request = plan_recovery_stage_completion(&assessed)?;

    assert_eq!(request.evidence(), assessed.evidence());
    assert_eq!(
        request.target(),
        RecoveryStageCompletionTarget::Catalog {
            generation: expected.generation(),
            length: expected.length(),
            digest: expected.digest(),
        }
    );
    Ok(())
}

#[test]
fn reusable_and_truncated_pool_stages_are_not_completion_requests() -> Result<(), Box<dyn Error>> {
    let segment = fixture(SEGMENT_HEX)?;
    let reusable_length = segment
        .len()
        .checked_sub(SEGMENT_SEAL_LENGTH)
        .ok_or("segment fixture is shorter than its seal")?;
    let reusable_bytes = segment
        .get(..reusable_length)
        .ok_or("reusable segment prefix is outside the fixture")?;
    let reusable = assessment(RecoveryStage::Segment, reusable_bytes)?;
    let truncated_catalog = assessment(RecoveryStage::Catalog, &[0_u8])?;

    for assessed in [&reusable, &truncated_catalog] {
        let error = plan_recovery_stage_completion(assessed)
            .err()
            .ok_or("incomplete stage produced a completion request")?;
        assert_eq!(
            error,
            RecoveryStageCompletionPlanError::NotComplete {
                stage: assessed.evidence().stage(),
            }
        );
    }
    Ok(())
}

#[test]
fn next_head_requires_its_dedicated_finalization_protocol() -> Result<(), Box<dyn Error>> {
    let bytes = fixture(HEAD_HEX)?;
    let assessed = assessment(RecoveryStage::NextHead, &bytes)?;

    let error = plan_recovery_stage_completion(&assessed)
        .err()
        .ok_or("next-head stage entered immutable-pool completion")?;

    assert_eq!(
        error,
        RecoveryStageCompletionPlanError::NotPoolStage {
            stage: RecoveryStage::NextHead,
        }
    );
    Ok(())
}
