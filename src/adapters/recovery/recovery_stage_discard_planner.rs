//! This module owns truncation-only recovery-stage discard planning.

use super::{
    RecoveryCatalogStage, RecoveryNextHeadStage, RecoverySegmentStage, RecoveryStageAssessment,
    RecoveryStageDiscardPlanError, RecoveryStageDiscardReason, RecoveryStageDiscardRequest,
};

/// Plans explicit discard only from an exact fingerprint-bound truncation.
///
/// The call performs no I/O or allocation. Reusable and complete stages remain
/// ineligible because their lawful recovery action is not implicit deletion.
///
/// # Errors
///
/// Returns [`RecoveryStageDiscardPlanError`] when the stage is not truncated.
pub const fn plan_recovery_stage_discard(
    assessment: &RecoveryStageAssessment<'_>,
) -> Result<RecoveryStageDiscardRequest, RecoveryStageDiscardPlanError> {
    let evidence = assessment.evidence();
    let reason = match assessment {
        RecoveryStageAssessment::Segment {
            state: RecoverySegmentStage::Truncated(reason),
            ..
        } => RecoveryStageDiscardReason::Segment(*reason),
        RecoveryStageAssessment::Catalog {
            state: RecoveryCatalogStage::HeaderTruncated { required, observed },
            ..
        } => RecoveryStageDiscardReason::CatalogHeader {
            required: *required,
            observed: *observed,
        },
        RecoveryStageAssessment::Catalog {
            state: RecoveryCatalogStage::BodyTruncated { expected, observed },
            ..
        } => RecoveryStageDiscardReason::CatalogBody {
            expected: *expected,
            observed: *observed,
        },
        RecoveryStageAssessment::NextHead {
            state: RecoveryNextHeadStage::Truncated { required, observed },
            ..
        } => RecoveryStageDiscardReason::NextHead {
            required: *required,
            observed: *observed,
        },
        RecoveryStageAssessment::Segment { .. }
        | RecoveryStageAssessment::Catalog { .. }
        | RecoveryStageAssessment::NextHead { .. } => {
            return Err(RecoveryStageDiscardPlanError::NotTruncated {
                stage: evidence.stage(),
            });
        }
    };
    Ok(RecoveryStageDiscardRequest::new(evidence, reason))
}
