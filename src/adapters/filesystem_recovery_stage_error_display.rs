//! This module owns filesystem recovery-stage error rendering.

use std::fmt;

use super::{FilesystemRecoveryStageError, RecoveryStageNamespacePhase};

impl fmt::Display for FilesystemRecoveryStageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MaterializeAddressSpace { .. }
            | Self::MaterializeAllocation { .. }
            | Self::Position { .. }
            | Self::Materialize { .. }
            | Self::PositionMismatch { .. } => fmt_materialization(self, formatter),
            Self::Namespace { .. }
            | Self::Open { .. }
            | Self::Inspect { .. }
            | Self::NonRegular { .. }
            | Self::MetadataAdmission { .. }
            | Self::Fingerprint { .. }
            | Self::Synchronize { .. }
            | Self::VerifyEntry { .. }
            | Self::Replaced { .. }
            | Self::LengthChanged { .. } => fmt_observation(self, formatter),
        }
    }
}

fn fmt_observation(
    error: &FilesystemRecoveryStageError,
    formatter: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    match error {
        FilesystemRecoveryStageError::Namespace { .. }
        | FilesystemRecoveryStageError::Open { .. }
        | FilesystemRecoveryStageError::Inspect { .. }
        | FilesystemRecoveryStageError::NonRegular { .. }
        | FilesystemRecoveryStageError::MetadataAdmission { .. } => {
            fmt_open_admission(error, formatter)
        }
        FilesystemRecoveryStageError::Fingerprint { .. }
        | FilesystemRecoveryStageError::Synchronize { .. }
        | FilesystemRecoveryStageError::VerifyEntry { .. }
        | FilesystemRecoveryStageError::Replaced { .. }
        | FilesystemRecoveryStageError::LengthChanged { .. } => fmt_verification(error, formatter),
        _ => fmt_materialization(error, formatter),
    }
}

fn fmt_open_admission(
    error: &FilesystemRecoveryStageError,
    formatter: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    match error {
        FilesystemRecoveryStageError::Namespace {
            stage,
            phase,
            source,
        } => write!(
            formatter,
            "{stage} namespace verification failed {phase}: {source}"
        ),
        FilesystemRecoveryStageError::Open { stage, source } => {
            write!(formatter, "failed to open recovery stage {stage}: {source}")
        }
        FilesystemRecoveryStageError::Inspect { stage, source } => {
            write!(
                formatter,
                "failed to inspect recovery stage {stage}: {source}"
            )
        }
        FilesystemRecoveryStageError::NonRegular { stage } => {
            write!(formatter, "recovery stage {stage} is not a regular file")
        }
        FilesystemRecoveryStageError::MetadataAdmission { stage, source } => {
            write!(
                formatter,
                "recovery stage {stage} metadata was refused: {source}"
            )
        }
        _ => fmt_materialization(error, formatter),
    }
}

fn fmt_verification(
    error: &FilesystemRecoveryStageError,
    formatter: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    match error {
        FilesystemRecoveryStageError::Fingerprint { stage, source } => {
            write!(
                formatter,
                "recovery stage {stage} fingerprint failed: {source}"
            )
        }
        FilesystemRecoveryStageError::Synchronize { stage, source } => {
            write!(
                formatter,
                "failed to synchronize recovery stage {stage}: {source}"
            )
        }
        FilesystemRecoveryStageError::VerifyEntry { stage, source } => write!(
            formatter,
            "failed to verify recovery stage entry {stage}: {source}"
        ),
        FilesystemRecoveryStageError::Replaced { stage } => {
            write!(formatter, "recovery stage {stage} changed file identity")
        }
        FilesystemRecoveryStageError::LengthChanged {
            stage,
            expected,
            observed,
        } => write!(
            formatter,
            "recovery stage {stage} changed length from {} to {observed}",
            expected.get()
        ),
        _ => fmt_materialization(error, formatter),
    }
}

fn fmt_materialization(
    error: &FilesystemRecoveryStageError,
    formatter: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    match error {
        FilesystemRecoveryStageError::MaterializeAddressSpace { stage, byte_count } => write!(
            formatter,
            "recovery stage {stage} byte count {byte_count} exceeds the host address space"
        ),
        FilesystemRecoveryStageError::MaterializeAllocation {
            stage, byte_count, ..
        } => write!(
            formatter,
            "cannot reserve {byte_count} bytes for recovery stage {stage}"
        ),
        FilesystemRecoveryStageError::Position { stage, source } => {
            write!(
                formatter,
                "cannot position recovery stage {stage}: {source}"
            )
        }
        FilesystemRecoveryStageError::Materialize {
            stage,
            expected,
            source,
        } => write!(
            formatter,
            "cannot materialize {} bytes from recovery stage {stage}: {source}",
            expected.get()
        ),
        FilesystemRecoveryStageError::PositionMismatch {
            stage,
            expected,
            observed,
        } => write!(
            formatter,
            "recovery stage {stage} position is {observed}, expected {}",
            expected.get()
        ),
        _ => fmt_observation(error, formatter),
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
