//! This module owns exact reusable-segment continuation execution.

use super::{
    RecoverySegmentResumeError, RecoverySegmentResumeRequest, RecoverySegmentResumeStorage,
    RecoverySegmentStage, RecoveryStage, RecoveryStageAssessment, StagedSegment,
    admit_recovery_stage_bytes, assess_recovery_stage,
};

/// Reopens one exact reusable prefix as the ordinary append-only stage state.
///
/// Storage is consumed so its writer authority remains owned by the returned
/// stage. The complete prefix is materialized once, re-admitted against prior
/// evidence, reclassified under the continuation policy, and used to rebuild
/// digest and duplicate-identity state before the first new write.
///
/// # Errors
///
/// Returns [`RecoverySegmentResumeError`] before returning a writable stage
/// when storage, evidence admission, semantic classification, or state
/// reconstruction fails.
pub fn execute_recovery_segment_resume<S>(
    storage: S,
    request: RecoverySegmentResumeRequest,
) -> Result<StagedSegment<S::Stage>, RecoverySegmentResumeError>
where
    S: RecoverySegmentResumeStorage,
{
    let opened = storage
        .open_reusable(request)
        .map_err(|source| RecoverySegmentResumeError::Open { source })?;
    let (stage, encoded) = opened.into_parts();
    let admitted = admit_recovery_stage_bytes(RecoveryStage::Segment, request.evidence(), &encoded)
        .map_err(|source| RecoverySegmentResumeError::Admission { source })?;
    let assessment = assess_recovery_stage(&admitted, request.policy())
        .map_err(|source| RecoverySegmentResumeError::Assessment { source })?;
    let RecoveryStageAssessment::Segment {
        state: RecoverySegmentStage::Reusable(reusable),
        ..
    } = assessment
    else {
        return Err(RecoverySegmentResumeError::NotReusable);
    };
    if reusable.record_count() != request.record_count() || reusable.length() != request.length() {
        return Err(RecoverySegmentResumeError::NotReusable);
    }
    StagedSegment::resume_admitted(stage, &encoded, request)
        .map_err(|source| RecoverySegmentResumeError::Rebuild { source })
}
