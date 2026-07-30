//! This module owns pure planning for reusable-segment continuation.

use super::{
    RecoverySegmentResumePlanError, RecoverySegmentResumeRequest, RecoverySegmentStage,
    RecoveryStageAssessment, SegmentReadPolicy,
};

/// Plans continuation only from an exact reusable `current.seg` assessment.
///
/// The returned request owns the observation evidence, complete-record count,
/// exact append boundary, and resource policy needed to re-admit the stage at
/// execution time.
///
/// # Errors
///
/// Returns [`RecoverySegmentResumePlanError`] when the assessment is not a
/// reusable segment prefix or the selected policy is already exceeded.
pub const fn plan_recovery_segment_resume(
    assessment: &RecoveryStageAssessment<'_>,
    policy: SegmentReadPolicy,
) -> Result<RecoverySegmentResumeRequest, RecoverySegmentResumePlanError> {
    let RecoveryStageAssessment::Segment { evidence, state } = assessment else {
        return Err(RecoverySegmentResumePlanError::NotSegment {
            stage: assessment.evidence().stage(),
        });
    };
    let RecoverySegmentStage::Reusable(reusable) = state else {
        return Err(RecoverySegmentResumePlanError::NotReusable {
            stage: evidence.stage(),
        });
    };
    if reusable.record_count() > policy.record_limit().get() {
        return Err(RecoverySegmentResumePlanError::RecordLimit {
            maximum: policy.record_limit(),
            observed: reusable.record_count(),
        });
    }
    Ok(RecoverySegmentResumeRequest::new(
        *evidence,
        reusable.record_count(),
        reusable.length(),
        policy,
    ))
}
