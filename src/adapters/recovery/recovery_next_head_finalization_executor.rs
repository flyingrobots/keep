//! This module owns ordered, idempotent recovery head finalization.

use super::{
    RecoveryNextHeadFinalizationError, RecoveryNextHeadFinalizationOutcome,
    RecoveryNextHeadFinalizationReadiness, RecoveryNextHeadFinalizationReceipt,
    RecoveryNextHeadFinalizationRequest, RecoveryNextHeadFinalizationStorage,
};

/// Finalizes one exact complete recovery next head.
///
/// A receipt is returned only after durable `HEAD` names the exact candidate
/// and the root directory has been synchronized. Retrying after replacement
/// but before directory synchronization re-admits the candidate and repeats the
/// root synchronization without replacing the head again. A ready candidate is
/// synchronized and reverified before atomic replacement.
/// Typed storage failures are boxed on the error path to keep the phase error
/// bounded without discarding expected or observed state.
///
/// # Errors
///
/// Returns [`RecoveryNextHeadFinalizationError`] without a receipt at the exact
/// verification, atomic replacement, or root synchronization phase.
pub fn execute_recovery_next_head_finalization(
    storage: &mut impl RecoveryNextHeadFinalizationStorage,
    request: RecoveryNextHeadFinalizationRequest,
) -> Result<RecoveryNextHeadFinalizationReceipt, RecoveryNextHeadFinalizationError> {
    let target = request.target();
    let readiness = storage.verify_current(request).map_err(|source| {
        RecoveryNextHeadFinalizationError::Verify {
            target,
            source: Box::new(source),
        }
    })?;
    let outcome = match readiness {
        RecoveryNextHeadFinalizationReadiness::Ready => {
            storage.synchronize_candidate(request).map_err(|source| {
                RecoveryNextHeadFinalizationError::SynchronizeCandidate {
                    evidence: request.evidence(),
                    source: Box::new(source),
                }
            })?;
            storage.replace_head(request).map_err(|source| {
                RecoveryNextHeadFinalizationError::Replace {
                    evidence: request.evidence(),
                    source: Box::new(source),
                }
            })?;
            RecoveryNextHeadFinalizationOutcome::Finalized
        }
        RecoveryNextHeadFinalizationReadiness::AlreadyFinalized => {
            RecoveryNextHeadFinalizationOutcome::AlreadyFinalized
        }
    };
    storage
        .synchronize_root()
        .map_err(|source| RecoveryNextHeadFinalizationError::SynchronizeRoot { target, source })?;
    Ok(RecoveryNextHeadFinalizationReceipt::new(request, outcome))
}
