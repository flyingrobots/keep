//! Reusable-segment recovery planning laws.

use std::error::Error;

use keep::{
    LayoutEntryLimit, RecoverySegmentResumePlanError, RecoveryStage, SegmentReadPolicy,
    SegmentRecordLimit, plan_recovery_segment_resume,
};

use super::{
    HEAD_HEX, SEGMENT_HEADER_LENGTH, SEGMENT_HEX, assessment, fixture, maximum_policy,
    reusable_prefix,
};
use crate::support::require_error;

#[test]
fn only_exact_reusable_segment_assessment_produces_a_resume_request() -> Result<(), Box<dyn Error>>
{
    let encoded = reusable_prefix()?;
    let assessed = assessment(RecoveryStage::Segment, &encoded)?;

    let request = plan_recovery_segment_resume(&assessed, maximum_policy())?;

    assert_eq!(request.evidence(), assessed.evidence());
    assert_eq!(request.record_count(), 1);
    assert_eq!(request.length().get(), u64::try_from(encoded.len())?);
    assert_eq!(request.policy(), maximum_policy());
    Ok(())
}

#[test]
fn truncated_segment_cannot_enter_reusable_resume() -> Result<(), Box<dyn Error>> {
    let encoded = reusable_prefix()?;
    let truncated = encoded
        .get(..SEGMENT_HEADER_LENGTH - 1)
        .ok_or("missing truncation")?;
    let assessed = assessment(RecoveryStage::Segment, truncated)?;

    let error = require_error(
        plan_recovery_segment_resume(&assessed, maximum_policy()),
        "truncated stage must not produce a resume request",
    )?;

    assert_eq!(
        error,
        RecoverySegmentResumePlanError::NotReusable {
            stage: RecoveryStage::Segment,
        }
    );
    Ok(())
}

#[test]
fn complete_segment_cannot_enter_reusable_resume() -> Result<(), Box<dyn Error>> {
    let encoded = fixture(SEGMENT_HEX)?;
    let assessed = assessment(RecoveryStage::Segment, &encoded)?;

    let error = require_error(
        plan_recovery_segment_resume(&assessed, maximum_policy()),
        "complete segment must not produce a resume request",
    )?;

    assert_eq!(
        error,
        RecoverySegmentResumePlanError::NotReusable {
            stage: RecoveryStage::Segment,
        }
    );
    Ok(())
}

#[test]
fn policy_below_the_admitted_record_count_is_refused() -> Result<(), Box<dyn Error>> {
    let encoded = reusable_prefix()?;
    let assessed = assessment(RecoveryStage::Segment, &encoded)?;
    let maximum = SegmentRecordLimit::new(0)?;
    let policy = SegmentReadPolicy::new(maximum, LayoutEntryLimit::MAXIMUM);

    let error = require_error(
        plan_recovery_segment_resume(&assessed, policy),
        "policy below the admitted prefix must be refused",
    )?;

    assert_eq!(
        error,
        RecoverySegmentResumePlanError::RecordLimit {
            maximum,
            observed: 1,
        }
    );
    Ok(())
}

#[test]
fn a_non_segment_stage_cannot_enter_reusable_resume() -> Result<(), Box<dyn Error>> {
    let encoded = fixture(HEAD_HEX)?;
    let assessed = assessment(RecoveryStage::NextHead, &encoded)?;

    let error = require_error(
        plan_recovery_segment_resume(&assessed, maximum_policy()),
        "head.next must not produce a segment resume request",
    )?;

    assert_eq!(
        error,
        RecoverySegmentResumePlanError::NotSegment {
            stage: RecoveryStage::NextHead,
        }
    );
    Ok(())
}
