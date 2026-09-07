//! This module owns exact complete-stage recovery planning.

use super::{
    RecoveryCatalogStage, RecoverySegmentStage, RecoveryStageAssessment,
    RecoveryStageCompletionPlanError, RecoveryStageCompletionRequest,
    RecoveryStageCompletionTarget,
};

/// Plans immutable-pool completion from one exact complete stage assessment.
///
/// The returned request owns only bounded evidence and validated coordinates;
/// it does not retain or allocate the assessed stage bytes.
///
/// # Errors
///
/// Returns [`RecoveryStageCompletionPlanError`] for incomplete segment or
/// catalog stages and for `head.next`, which has a dedicated finalization
/// protocol.
pub const fn plan_recovery_stage_completion(
    assessment: &RecoveryStageAssessment<'_>,
) -> Result<RecoveryStageCompletionRequest, RecoveryStageCompletionPlanError> {
    let evidence = assessment.evidence();
    let target = match assessment {
        RecoveryStageAssessment::Segment {
            state: RecoverySegmentStage::Complete(segment),
            ..
        } => RecoveryStageCompletionTarget::Segment {
            digest: segment.digest(),
        },
        RecoveryStageAssessment::Catalog {
            state: RecoveryCatalogStage::Complete(catalog),
            ..
        } => RecoveryStageCompletionTarget::Catalog {
            generation: catalog.generation(),
            length: catalog.length(),
            digest: catalog.digest(),
        },
        RecoveryStageAssessment::Segment { .. } | RecoveryStageAssessment::Catalog { .. } => {
            return Err(RecoveryStageCompletionPlanError::NotComplete {
                stage: evidence.stage(),
            });
        }
        RecoveryStageAssessment::NextHead { .. } => {
            return Err(RecoveryStageCompletionPlanError::NotPoolStage {
                stage: evidence.stage(),
            });
        }
    };
    Ok(RecoveryStageCompletionRequest::new(evidence, target))
}
