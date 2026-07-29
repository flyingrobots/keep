//! This module owns deterministic crash-matrix execution failures.

use std::error::Error;
use std::fmt;
use std::io;
use std::time::Duration;

use xtask::DurabilityCrashCaseError;

pub(crate) enum DurabilityCrashMatrixError {
    ChildExitedEarly {
        code: Option<i32>,
    },
    ChildSurvivedTermination {
        code: Option<i32>,
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
    StateMismatch,
    Timeout {
        duration: Duration,
    },
    UnknownPoint(String),
    UnknownPosition(String),
    Usage,
}

impl DurabilityCrashMatrixError {
    pub(crate) const fn io(action: &'static str, source: io::Error) -> Self {
        Self::Io { action, source }
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
            Self::InvalidCase(error) => write!(formatter, "invalid crash case: {error}"),
            Self::InvalidPointEncoding => formatter.write_str("crash point is not valid Unicode"),
            Self::InvalidPositionEncoding => {
                formatter.write_str("crash position is not valid Unicode")
            }
            Self::InvalidReadinessSignal { observed } => {
                write!(formatter, "crash child sent readiness byte {observed}")
            }
            Self::Io { action, .. } => write!(formatter, "cannot {action}"),
            Self::StateMismatch => {
                formatter.write_str("crash child durable marker does not match its case")
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
        }
    }
}

impl Error for DurabilityCrashMatrixError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidCase(error) => Some(error),
            Self::Io { source, .. } => Some(source),
            Self::ChildExitedEarly { .. }
            | Self::ChildSurvivedTermination { .. }
            | Self::InvalidPointEncoding
            | Self::InvalidPositionEncoding
            | Self::InvalidReadinessSignal { .. }
            | Self::StateMismatch
            | Self::Timeout { .. }
            | Self::UnknownPoint(_)
            | Self::UnknownPosition(_)
            | Self::Usage => None,
        }
    }
}
