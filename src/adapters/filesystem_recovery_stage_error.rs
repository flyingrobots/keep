//! This module owns capability-relative recovery-stage observation failures.

use std::collections::TryReserveError;
use std::error::Error;
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
    /// The admitted stage length cannot be represented for materialization.
    MaterializeAddressSpace {
        /// Fixed stage being materialized.
        stage: RecoveryStage,
        /// Admitted byte count that exceeds the host address space.
        byte_count: u64,
    },
    /// Memory for exact bounded materialization could not be reserved.
    MaterializeAllocation {
        /// Fixed stage being materialized.
        stage: RecoveryStage,
        /// Exact admitted byte count requested.
        byte_count: u64,
        /// Allocation reservation failure.
        source: TryReserveError,
    },
    /// The writable stage could not be rewound or its position inspected.
    Position {
        /// Fixed stage being positioned.
        stage: RecoveryStage,
        /// Exact underlying positioning failure.
        source: io::Error,
    },
    /// Exact prefix materialization failed.
    Materialize {
        /// Fixed stage being materialized.
        stage: RecoveryStage,
        /// Exact admitted byte count requested.
        expected: RecoveryStageLength,
        /// Exact underlying read failure.
        source: io::Error,
    },
    /// The writable handle is not positioned at the admitted append boundary.
    PositionMismatch {
        /// Fixed stage being positioned.
        stage: RecoveryStage,
        /// Exact admitted append boundary.
        expected: RecoveryStageLength,
        /// Observed writable-handle position.
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

impl Error for FilesystemRecoveryStageError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Namespace { source, .. } => Some(source),
            Self::Open { source, .. }
            | Self::Inspect { source, .. }
            | Self::Synchronize { source, .. }
            | Self::VerifyEntry { source, .. }
            | Self::Position { source, .. }
            | Self::Materialize { source, .. } => Some(source),
            Self::MetadataAdmission { source, .. } => Some(source),
            Self::Fingerprint { source, .. } => Some(source),
            Self::MaterializeAllocation { source, .. } => Some(source),
            Self::NonRegular { .. }
            | Self::Replaced { .. }
            | Self::LengthChanged { .. }
            | Self::MaterializeAddressSpace { .. }
            | Self::PositionMismatch { .. } => None,
        }
    }
}
