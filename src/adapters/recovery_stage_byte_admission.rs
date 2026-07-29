//! This module owns admission of materialized bytes against prior stage evidence.

use super::{
    AdmittedRecoveryStageBytes, RecoveryStage, RecoveryStageByteAdmissionError,
    RecoveryStageEvidence, RecoveryStageMetadata, fingerprint_recovery_stage,
};

/// Admits complete materialized bytes only when they equal prior stage evidence.
///
/// The call performs no I/O or allocation. It checks the canonical-name stage
/// and exact length before recomputing the versioned bounded fingerprint.
///
/// # Errors
///
/// Returns [`RecoveryStageByteAdmissionError`] on stage, length, protocol
/// maximum, or fingerprint disagreement.
pub fn admit_recovery_stage_bytes(
    expected_stage: RecoveryStage,
    evidence: RecoveryStageEvidence,
    encoded: &[u8],
) -> Result<AdmittedRecoveryStageBytes<'_>, RecoveryStageByteAdmissionError> {
    if evidence.stage() != expected_stage {
        return Err(RecoveryStageByteAdmissionError::StageMismatch {
            expected: expected_stage,
            observed: evidence.stage(),
        });
    }
    let observed = u64::try_from(encoded.len()).map_err(|_| {
        RecoveryStageByteAdmissionError::AddressSpace {
            observed: encoded.len(),
        }
    })?;
    if evidence.length().get() != observed {
        return Err(RecoveryStageByteAdmissionError::LengthMismatch {
            stage: expected_stage,
            expected: evidence.length(),
            observed,
        });
    }
    let metadata = RecoveryStageMetadata::new(expected_stage, observed)
        .map_err(|source| RecoveryStageByteAdmissionError::Metadata { source })?;
    let recomputed = fingerprint_recovery_stage(metadata, encoded).map_err(|source| {
        RecoveryStageByteAdmissionError::Fingerprint {
            stage: expected_stage,
            source,
        }
    })?;
    if recomputed.fingerprint() != evidence.fingerprint() {
        return Err(RecoveryStageByteAdmissionError::FingerprintMismatch {
            stage: expected_stage,
            expected: evidence.fingerprint(),
            observed: recomputed.fingerprint(),
        });
    }
    Ok(AdmittedRecoveryStageBytes::new(evidence, encoded))
}
