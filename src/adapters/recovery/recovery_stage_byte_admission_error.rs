//! This module owns recovery-stage byte-admission failures.

use std::error::Error;
use std::fmt;

use super::{
    RecoveryStage, RecoveryStageFingerprint, RecoveryStageFingerprintError, RecoveryStageLength,
    RecoveryStageMetadataError,
};

/// Why materialized bytes did not match prior fixed-stage evidence.
#[derive(Debug)]
pub enum RecoveryStageByteAdmissionError {
    /// The canonical-name-selected stage differs from the evidence.
    StageMismatch {
        /// Stage selected by the canonical inventory name.
        expected: RecoveryStage,
        /// Stage carried by the prior evidence.
        observed: RecoveryStage,
    },
    /// The supplied slice length cannot fit the protocol coordinate.
    AddressSpace {
        /// Host byte count that could not be represented.
        observed: usize,
    },
    /// The supplied byte length differs from prior evidence.
    LengthMismatch {
        /// Fixed stage being admitted.
        stage: RecoveryStage,
        /// Previously observed exact length.
        expected: RecoveryStageLength,
        /// Supplied byte count.
        observed: u64,
    },
    /// The supplied length violates the selected stage's protocol maximum.
    Metadata {
        /// Exact metadata-admission refusal.
        source: RecoveryStageMetadataError,
    },
    /// Recomputing the bounded versioned fingerprint failed.
    Fingerprint {
        /// Fixed stage being admitted.
        stage: RecoveryStage,
        /// Exact fingerprinting refusal.
        source: RecoveryStageFingerprintError,
    },
    /// The supplied bytes differ from the prior observation.
    FingerprintMismatch {
        /// Fixed stage being admitted.
        stage: RecoveryStage,
        /// Previously observed fingerprint.
        expected: RecoveryStageFingerprint,
        /// Fingerprint of the supplied complete bytes.
        observed: RecoveryStageFingerprint,
    },
}

impl fmt::Display for RecoveryStageByteAdmissionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::StageMismatch { expected, observed } => write!(
                formatter,
                "recovery stage evidence names {observed}, expected {expected}"
            ),
            Self::AddressSpace { observed } => write!(
                formatter,
                "recovery stage length {observed} does not fit the protocol coordinate"
            ),
            Self::LengthMismatch {
                stage,
                expected,
                observed,
            } => write!(
                formatter,
                "{stage} length {observed} differs from observed length {}",
                expected.get()
            ),
            Self::Metadata { source } => {
                write!(formatter, "recovery stage metadata was refused: {source}")
            }
            Self::Fingerprint { stage, source } => {
                write!(
                    formatter,
                    "{stage} fingerprint recomputation failed: {source}"
                )
            }
            Self::FingerprintMismatch { stage, .. } => {
                write!(formatter, "{stage} fingerprint differs from prior evidence")
            }
        }
    }
}

impl Error for RecoveryStageByteAdmissionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Metadata { source } => Some(source),
            Self::Fingerprint { source, .. } => Some(source),
            Self::StageMismatch { .. }
            | Self::AddressSpace { .. }
            | Self::LengthMismatch { .. }
            | Self::FingerprintMismatch { .. } => None,
        }
    }
}
