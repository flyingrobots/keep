//! This module owns bounded streaming recovery-stage observation.

use std::io::{self, Read};

use super::{
    RecoveryStage, RecoveryStageEvidence, RecoveryStageFingerprint, RecoveryStageFingerprintError,
    RecoveryStageLength, RecoveryStageMetadata, framed_blake3,
};

const DOMAIN: &[u8] = b"KEEP:RECOVERY:STAGE\0";
const BUFFER_LENGTH: usize = 8_192;

/// Reads and fingerprints one complete fixed recovery stage without retaining
/// its bytes.
///
/// The reader is offered at most the name-selected maximum plus one byte. The
/// returned evidence binds the exact observed length and version-1
/// domain-separated digest.
///
/// # Errors
///
/// Returns [`RecoveryStageFingerprintError`] for oversized stream evidence, a
/// broken reader contract, checked length failure, or underlying I/O failure.
pub fn fingerprint_recovery_stage<R: Read>(
    metadata: RecoveryStageMetadata,
    mut reader: R,
) -> Result<RecoveryStageEvidence, RecoveryStageFingerprintError> {
    let stage = metadata.stage();
    let maximum = stage.maximum_length();
    read_bounded(stage, maximum, &mut reader)
}

fn read_bounded<R: Read>(
    stage: RecoveryStage,
    maximum: u64,
    reader: &mut R,
) -> Result<RecoveryStageEvidence, RecoveryStageFingerprintError> {
    let limit = maximum
        .checked_add(1)
        .ok_or(RecoveryStageFingerprintError::LengthOverflow {
            stage,
            offset: maximum,
            increment: 1,
        })?;
    let mut fingerprint_state = framed_blake3::State::new(DOMAIN);
    let mut buffer = [0_u8; BUFFER_LENGTH];
    let buffer_length = u64::try_from(BUFFER_LENGTH).map_err(|_| {
        RecoveryStageFingerprintError::PlatformBufferLength {
            stage,
            observed: BUFFER_LENGTH,
        }
    })?;
    let mut offset = 0_u64;
    loop {
        let remaining =
            limit
                .checked_sub(offset)
                .ok_or(RecoveryStageFingerprintError::LengthOverflow {
                    stage,
                    offset,
                    increment: 0,
                })?;
        let offered_u64 = remaining.min(buffer_length);
        let offered = usize::try_from(offered_u64).map_err(|_| {
            RecoveryStageFingerprintError::PlatformLength {
                stage,
                observed: offered_u64,
            }
        })?;
        let target =
            buffer
                .get_mut(..offered)
                .ok_or(RecoveryStageFingerprintError::PlatformLength {
                    stage,
                    observed: offered_u64,
                })?;
        let count = match reader.read(target) {
            Ok(0) => break,
            Ok(count) => count,
            Err(source) if source.kind() == io::ErrorKind::Interrupted => continue,
            Err(source) => {
                return Err(RecoveryStageFingerprintError::Read {
                    stage,
                    offset,
                    source,
                });
            }
        };
        if count > offered {
            return Err(RecoveryStageFingerprintError::ReaderContract {
                stage,
                offset,
                offered,
                observed: count,
            });
        }
        let increment =
            u64::try_from(count).map_err(|_| RecoveryStageFingerprintError::PlatformLength {
                stage,
                observed: offered_u64,
            })?;
        let observed =
            offset
                .checked_add(increment)
                .ok_or(RecoveryStageFingerprintError::LengthOverflow {
                    stage,
                    offset,
                    increment,
                })?;
        if observed > maximum {
            return Err(RecoveryStageFingerprintError::EvidenceOversized {
                stage,
                maximum,
                observed_at_least: observed,
            });
        }
        let bytes = buffer
            .get(..count)
            .ok_or(RecoveryStageFingerprintError::ReaderContract {
                stage,
                offset,
                offered,
                observed: count,
            })?;
        fingerprint_state.update(bytes);
        offset = observed;
    }
    Ok(RecoveryStageEvidence::new(
        stage,
        RecoveryStageLength::from_validated(offset),
        RecoveryStageFingerprint::from_validated(fingerprint_state.finalize(offset)),
    ))
}
