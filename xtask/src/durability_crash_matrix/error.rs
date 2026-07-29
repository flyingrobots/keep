//! This module owns deterministic crash-matrix execution failures.

use std::error::Error;
use std::fmt;
use std::io;
use std::time::Duration;

use keep::WriterLockAcquireError;
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
    Io {
        action: &'static str,
        source: io::Error,
    },
    MissingActiveFile,
    NonUnicodeStatePath,
    PointSequenceMismatch {
        point: DurabilityCrashPoint,
    },
    StateMismatch,
    Timeout {
        duration: Duration,
    },
    UnknownPoint(String),
    UnknownPosition(String),
    Usage,
    Verification {
        phase: &'static str,
        source: Box<dyn Error>,
    },
    WriterLock(WriterLockAcquireError),
}

impl DurabilityCrashMatrixError {
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

impl fmt::Debug for DurabilityCrashMatrixError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, formatter)
    }
}

impl fmt::Display for DurabilityCrashMatrixError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Case {
                point,
                position,
                source,
            } => write!(
                formatter,
                "{} {}: {source}",
                point.identifier(),
                position.identifier()
            ),
            Self::ChildExitedEarly { code } => {
                write!(
                    formatter,
                    "crash child exited before readiness with code {code:?}"
                )
            }
            Self::ChildSurvivedTermination { code } => {
                write!(
                    formatter,
                    "crash child survived termination with code {code:?}"
                )
            }
            Self::Fixture { artifact, .. } => {
                write!(formatter, "cannot decode {artifact} crash fixture")
            }
            Self::FixtureLength {
                artifact,
                expected,
                observed,
            } => write!(
                formatter,
                "{artifact} crash fixture has length {observed}, expected {expected}"
            ),
            Self::FixtureRange => formatter.write_str("crash fixture range is invalid"),
            Self::FixtureTerminator { artifact } => {
                write!(
                    formatter,
                    "{artifact} crash fixture lacks its final line feed"
                )
            }
            Self::InvalidCase(error) => write!(formatter, "invalid crash case: {error}"),
            Self::InvalidPointEncoding => formatter.write_str("crash point is not valid Unicode"),
            Self::InvalidPositionEncoding => {
                formatter.write_str("crash position is not valid Unicode")
            }
            Self::InvalidReadinessSignal { observed } => {
                write!(formatter, "crash child sent readiness byte {observed}")
            }
            Self::Io { action, .. } => write!(formatter, "cannot {action}"),
            Self::MissingActiveFile => {
                formatter.write_str("crash sequence has no active staged artifact")
            }
            Self::NonUnicodeStatePath => {
                formatter.write_str("post-crash store path is not valid Unicode")
            }
            Self::PointSequenceMismatch { point } => write!(
                formatter,
                "{} is outside the selected crash sequence",
                point.identifier()
            ),
            Self::StateMismatch => {
                formatter.write_str("post-crash store state does not match its case")
            }
            Self::Timeout { duration } => {
                write!(formatter, "crash child exceeded its {duration:?} deadline")
            }
            Self::UnknownPoint(point) => write!(formatter, "unknown crash point `{point}`"),
            Self::UnknownPosition(position) => {
                write!(formatter, "unknown crash position `{position}`")
            }
            Self::Usage => formatter.write_str(
                "usage: cargo xtask durability-crash-matrix \
                 --case <KEEP-CRASH-NNN> <before|during|after>",
            ),
            Self::Verification { phase, source } => {
                write!(
                    formatter,
                    "post-crash verification failed while attempting to {phase}: {source}"
                )
            }
            Self::WriterLock(source) => {
                write!(formatter, "cannot acquire crash-case writer lock: {source}")
            }
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
            Self::WriterLock(source) => Some(source),
            Self::ChildExitedEarly { .. }
            | Self::ChildSurvivedTermination { .. }
            | Self::FixtureLength { .. }
            | Self::FixtureRange
            | Self::FixtureTerminator { .. }
            | Self::InvalidPointEncoding
            | Self::InvalidPositionEncoding
            | Self::InvalidReadinessSignal { .. }
            | Self::MissingActiveFile
            | Self::NonUnicodeStatePath
            | Self::PointSequenceMismatch { .. }
            | Self::StateMismatch
            | Self::Timeout { .. }
            | Self::UnknownPoint(_)
            | Self::UnknownPosition(_)
            | Self::Usage => None,
        }
    }
}
