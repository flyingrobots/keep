//! This module owns name-selected dispatch of admitted recovery-stage bytes.

use super::{
    AdmittedRecoveryStageBytes, RecoveryStage, RecoveryStageAssessment,
    RecoveryStageAssessmentError, SegmentReadPolicy, classify_recovery_catalog_stage,
    classify_recovery_next_head_stage, classify_recovery_segment_stage,
};

/// Classifies fingerprint-bound bytes through their only lawful stage grammar.
///
/// The call performs no I/O or content copy. Segment assessment may allocate a
/// duplicate-identity index bounded by `segment_policy.record_limit()`;
/// catalog and candidate-head assessment allocate nothing.
///
/// # Errors
///
/// Returns [`RecoveryStageAssessmentError`] with the exact name-selected
/// semantic classifier refusal.
pub fn assess_recovery_stage<'a>(
    admitted: &AdmittedRecoveryStageBytes<'a>,
    segment_policy: SegmentReadPolicy,
) -> Result<RecoveryStageAssessment<'a>, RecoveryStageAssessmentError> {
    let evidence = admitted.evidence();
    match admitted.stage() {
        RecoveryStage::Segment => {
            classify_recovery_segment_stage(admitted.encoded(), segment_policy)
                .map(|state| RecoveryStageAssessment::Segment { evidence, state })
                .map_err(|source| RecoveryStageAssessmentError::Segment { source })
        }
        RecoveryStage::Catalog => classify_recovery_catalog_stage(admitted.encoded())
            .map(|state| RecoveryStageAssessment::Catalog { evidence, state })
            .map_err(|source| RecoveryStageAssessmentError::Catalog { source }),
        RecoveryStage::NextHead => classify_recovery_next_head_stage(admitted.encoded())
            .map(|state| RecoveryStageAssessment::NextHead { evidence, state })
            .map_err(|source| RecoveryStageAssessmentError::NextHead { source }),
    }
}
