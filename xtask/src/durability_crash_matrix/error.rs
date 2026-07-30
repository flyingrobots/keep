//! This module owns deterministic crash-matrix execution failures.

mod display;

use std::collections::BTreeSet;
use std::error::Error;
use std::io;
use std::time::Duration;

use xtask::protocol_admission::HexError;
use xtask::{
    DurabilityCrashCase, DurabilityCrashCaseError, DurabilityCrashPoint, DurabilityCrashPosition,
};

pub(crate) enum DurabilityCrashMatrixError {
    Case {
        point: DurabilityCrashPoint,
        position: DurabilityCrashPosition,
        source: Box<Self>,
    },
    ArtifactBytesMismatch {
        artifact: &'static str,
        expected_length: usize,
        observed_length: usize,
        offset: usize,
        expected: Option<u8>,
        observed: Option<u8>,
    },
    ArtifactClassificationMismatch {
        expected: &'static str,
        observed: &'static str,
    },
    ChildExitedEarly {
        code: Option<i32>,
    },
    ChildSurvivedTermination {
        code: Option<i32>,
    },
    Fixture {
        artifact: &'static str,
        source: HexError,
    },
    FixtureLength {
        artifact: &'static str,
        expected: usize,
        observed: usize,
    },
    FixtureRange,
    FixtureTerminator {
        artifact: &'static str,
    },
    InvalidCase(DurabilityCrashCaseError),
    InvalidPointEncoding,
    InvalidPositionEncoding,
    InvalidReadinessSignal {
        observed: u8,
    },
    InventoryMismatch {
        expected: BTreeSet<String>,
        observed: BTreeSet<String>,
    },
    Io {
        action: &'static str,
        source: io::Error,
    },
    HardLinkIdentityMismatch {
        source: &'static str,
        target: &'static str,
        source_device: u64,
        source_inode: u64,
        target_device: u64,
        target_inode: u64,
    },
    MissingVisibleRecord {
        record: &'static str,
    },
    NonUnicodeStatePath,
    PointSequenceMismatch {
        point: DurabilityCrashPoint,
    },
    RepeatedInventoryPath {
        path: String,
    },
    SnapshotGenerationMismatch {
        expected: u64,
        observed: u64,
    },
    Timeout {
        duration: Duration,
    },
    UnexpectedArtifactKind {
        artifact: &'static str,
        expected: &'static str,
        observed: &'static str,
    },
    UnknownPoint(String),
    UnknownPosition(String),
    Usage,
    Verification {
        phase: &'static str,
        source: Box<dyn Error>,
    },
}

impl DurabilityCrashMatrixError {
    pub(crate) fn artifact_bytes(artifact: &'static str, expected: &[u8], observed: &[u8]) -> Self {
        let offset = expected
            .iter()
            .zip(observed)
            .position(|(expected, observed)| expected != observed)
            .unwrap_or_else(|| expected.len().min(observed.len()));
        Self::ArtifactBytesMismatch {
            artifact,
            expected_length: expected.len(),
            observed_length: observed.len(),
            offset,
            expected: expected.get(offset).copied(),
            observed: observed.get(offset).copied(),
        }
    }

    pub(crate) const fn io(action: &'static str, source: io::Error) -> Self {
        Self::Io { action, source }
    }

    pub(crate) fn at_case(self, case: DurabilityCrashCase) -> Self {
        Self::Case {
            point: case.point(),
            position: case.position(),
            source: Box::new(self),
        }
    }
}

impl Error for DurabilityCrashMatrixError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Case { source, .. } => Some(source.as_ref()),
            Self::Fixture { source, .. } => Some(source),
            Self::InvalidCase(error) => Some(error),
            Self::Io { source, .. } => Some(source),
            Self::Verification { source, .. } => Some(source.as_ref()),
            Self::ArtifactBytesMismatch { .. }
            | Self::ArtifactClassificationMismatch { .. }
            | Self::ChildExitedEarly { .. }
            | Self::ChildSurvivedTermination { .. }
            | Self::FixtureLength { .. }
            | Self::FixtureRange
            | Self::FixtureTerminator { .. }
            | Self::InvalidPointEncoding
            | Self::InvalidPositionEncoding
            | Self::InvalidReadinessSignal { .. }
            | Self::InventoryMismatch { .. }
            | Self::HardLinkIdentityMismatch { .. }
            | Self::MissingVisibleRecord { .. }
            | Self::NonUnicodeStatePath
            | Self::PointSequenceMismatch { .. }
            | Self::RepeatedInventoryPath { .. }
            | Self::SnapshotGenerationMismatch { .. }
            | Self::Timeout { .. }
            | Self::UnexpectedArtifactKind { .. }
            | Self::UnknownPoint(_)
            | Self::UnknownPosition(_)
            | Self::Usage => None,
        }
    }
}
