//! This module owns ordered, idempotent complete-stage recovery.

use super::{
    RecoveryStageCompletionError, RecoveryStageCompletionReceipt, RecoveryStageCompletionRequest,
    RecoveryStageCompletionStorage,
};

/// Completes one exact stage into a durable, verified immutable orphan.
///
/// The operation never creates, replaces, or finalizes a catalog head. A
/// receipt is returned only after the immutable pool is synchronized, the exact
/// fixed stage is absent, and staging is synchronized.
///
/// # Errors
///
/// Returns [`RecoveryStageCompletionError`] without a receipt at the exact
/// failed link, verification, pool-sync, removal, or staging-sync phase.
pub fn execute_recovery_stage_completion(
    storage: &mut impl RecoveryStageCompletionStorage,
    request: RecoveryStageCompletionRequest,
) -> Result<RecoveryStageCompletionReceipt, RecoveryStageCompletionError> {
    let target = request.target();
    let pool = request.pool();
    let stage = request.evidence().stage();
    let synchronization_outcome = storage
        .synchronize_stage_if_present(request)
        .map_err(|source| RecoveryStageCompletionError::SynchronizeStage { stage, source })?;
    let pool_outcome = storage
        .link_stage_or_admit_pool(request)
        .map_err(|source| RecoveryStageCompletionError::LinkOrAdmit { target, source })?;
    storage
        .verify_pool(request)
        .map_err(|source| RecoveryStageCompletionError::VerifyPool { target, source })?;
    storage
        .synchronize_pool(pool)
        .map_err(|source| RecoveryStageCompletionError::SynchronizePool { pool, source })?;
    let stage_outcome = storage
        .remove_stage_if_matching(request.evidence())
        .map_err(|source| RecoveryStageCompletionError::RemoveStage { source })?;
    storage
        .synchronize_staging()
        .map_err(|source| RecoveryStageCompletionError::SynchronizeStaging { stage, source })?;
    Ok(RecoveryStageCompletionReceipt::new(
        request,
        synchronization_outcome,
        pool_outcome,
        stage_outcome,
    ))
}
