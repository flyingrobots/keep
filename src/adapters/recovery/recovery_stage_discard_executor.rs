//! This module owns ordered, idempotent truncated-stage discard execution.

use super::{
    RecoveryStageDiscardError, RecoveryStageDiscardReceipt, RecoveryStageDiscardRequest,
    RecoveryStageDiscardStorage,
};

/// Removes one exact truncated stage and durably synchronizes its parent.
///
/// An already absent canonical name still requires parent synchronization.
/// The call performs no allocation beyond work owned by the storage adapter.
///
/// # Errors
///
/// Returns [`RecoveryStageDiscardError`] without a receipt when exact-evidence
/// removal, absence admission, or parent synchronization fails. Synchronization
/// errors retain whether this call removed the stage or found it already absent.
/// The failed sync leaves directory durability unconfirmed; it does not roll back
/// removal. A removal error makes no claim that the capability had no effects.
pub fn execute_recovery_stage_discard(
    storage: &mut impl RecoveryStageDiscardStorage,
    request: RecoveryStageDiscardRequest,
) -> Result<RecoveryStageDiscardReceipt, RecoveryStageDiscardError> {
    let stage = request.stage();
    let outcome = storage
        .remove_if_matching(request.evidence())
        .map_err(|source| RecoveryStageDiscardError::Remove { source })?;
    storage
        .synchronize_parent(stage.parent())
        .map_err(|source| RecoveryStageDiscardError::Synchronize {
            stage,
            outcome,
            source,
        })?;
    Ok(RecoveryStageDiscardReceipt::new(request, outcome))
}
