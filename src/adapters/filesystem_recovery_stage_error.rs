//! This module owns capability-relative recovery-stage observation failures.

use std::error::Error;
use std::fmt;
use std::io;

use super::{
    RecoveryInventoryError, RecoveryStage, RecoveryStageFingerprintError, RecoveryStageLength,
    RecoveryStageMetadataError,
};

/// Why one filesystem recovery stage could not produce exact evidence.
#[derive(Debug)]
pub enum FilesystemRecoveryStageError {
    /// A pinned recovery namespace failed verification.
    Namespace {
        /// Fixed stage being observed.
        stage: RecoveryStage,
        /// Whether verification failed before or after observation.
        phase: RecoveryStageNamespacePhase,
        /// Underlying namespace refusal.
        source: RecoveryInventoryError,
    },
    /// The fixed stage could not be opened without following links.
    Open {
        /// Fixed stage being observed.
        stage: RecoveryStage,
        /// Underlying open failure.
        source: io::Error,
    },
    /// Metadata could not be read from an opened stage handle.
    Inspect {
        /// Fixed stage being observed.
        stage: RecoveryStage,
        /// Underlying metadata failure.
        source: io::Error,
    },
    /// The fixed stage exists but is not a regular file.
    NonRegular {
        /// Fixed stage being observed.
        stage: RecoveryStage,
    },
    /// The stage metadata length exceeds its name-selected maximum.
    MetadataAdmission {
        /// Fixed stage being observed.
        stage: RecoveryStage,
        /// Exact metadata-admission refusal.
        source: RecoveryStageMetadataError,
    },
    /// Bounded streaming fingerprinting failed.
    Fingerprint {
        /// Fixed stage being observed.
        stage: RecoveryStage,
        /// Exact streaming refusal.
        source: RecoveryStageFingerprintError,
    },
    /// The exact complete stage could not be synchronized.
    Synchronize {
        /// Fixed stage being synchronized.
        stage: RecoveryStage,
        /// Exact staged-file synchronization failure.
        source: io::Error,
    },
    /// The fixed stage entry could not be reopened for identity verification.
    VerifyEntry {
        /// Fixed stage being observed.
        stage: RecoveryStage,
        /// Underlying verification-open failure.
        source: io::Error,
    },
    /// The fixed stage entry no longer names the opened file.
    Replaced {
        /// Fixed stage being observed.
        stage: RecoveryStage,
    },
    /// The file length changed while evidence was collected.
    LengthChanged {
        /// Fixed stage being observed.
        stage: RecoveryStage,
        /// Length admitted before reading.
        expected: RecoveryStageLength,
        /// Length observed during or after reading.
        observed: u64,
    },
}

/// Namespace-verification position around stage observation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecoveryStageNamespacePhase {
    /// Verification before opening the stage.
    BeforeObservation,
    /// Verification after entry and handle verification.
    AfterObservation,
}

impl fmt::Display for FilesystemRecoveryStageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Namespace {
                stage,
                phase,
                source,
            } => write!(
                formatter,
                "{stage} namespace verification failed {phase}: {source}"
            ),
            Self::Open { stage, source } => {
                write!(formatter, "failed to open recovery stage {stage}: {source}")
            }
            Self::Inspect { stage, source } => {
                write!(
                    formatter,
                    "failed to inspect recovery stage {stage}: {source}"
                )
            }
            Self::NonRegular { stage } => {
                write!(formatter, "recovery stage {stage} is not a regular file")
            }
            Self::MetadataAdmission { stage, source, .. } => write!(
                formatter,
                "recovery stage {stage} metadata was refused: {source}"
            ),
            Self::Fingerprint { stage, source, .. } => write!(
                formatter,
                "recovery stage {stage} fingerprint failed: {source}"
            ),
            Self::Synchronize { stage, source } => {
                write!(
                    formatter,
                    "failed to synchronize recovery stage {stage}: {source}"
                )
            }
            Self::VerifyEntry { stage, source } => write!(
                formatter,
                "failed to verify recovery stage entry {stage}: {source}"
            ),
            Self::Replaced { stage } => {
                write!(formatter, "recovery stage {stage} changed file identity")
            }
            Self::LengthChanged {
                stage,
                expected,
                observed,
            } => write!(
                formatter,
                "recovery stage {stage} changed length from {} to {observed}",
                expected.get()
            ),
        }
    }
}

impl fmt::Display for RecoveryStageNamespacePhase {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::BeforeObservation => "before observation",
            Self::AfterObservation => "after observation",
        })
    }
}

impl Error for FilesystemRecoveryStageError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Namespace { source, .. } => Some(source),
            Self::Open { source, .. }
            | Self::Inspect { source, .. }
            | Self::Synchronize { source, .. }
            | Self::VerifyEntry { source, .. } => Some(source),
            Self::MetadataAdmission { source, .. } => Some(source),
            Self::Fingerprint { source, .. } => Some(source),
            Self::NonRegular { .. } | Self::Replaced { .. } | Self::LengthChanged { .. } => None,
        }
    }
}
