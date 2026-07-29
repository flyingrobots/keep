//! This module owns typed bounded-process failures.

use std::error::Error;
use std::fmt;
use std::io;
use std::time::Duration;

/// A typed failure from synchronous, bounded child-process execution.
pub(crate) enum ProcessError {
    /// Reader or cleanup collection found another failure after the primary one.
    Additional {
        primary: Box<Self>,
        additional: Box<Self>,
    },
    /// Process-group termination or child reaping failed after a primary error.
    Cleanup {
        primary: Box<Self>,
        action: &'static str,
        source: io::Error,
    },
    /// A named operating-system process action failed.
    Io {
        program: &'static str,
        action: &'static str,
        source: io::Error,
    },
    /// A terminal signal interrupted the complete child operation.
    Interrupted {
        program: &'static str,
        signal: &'static str,
    },
    /// A child configured for capture did not expose the requested pipe.
    MissingStream {
        program: &'static str,
        stream: &'static str,
    },
    /// A captured stream exceeded its fixed retained-byte limit.
    OutputLimit {
        program: &'static str,
        stream: &'static str,
        maximum: usize,
    },
    /// A dedicated output-reader thread stopped by panicking.
    ReaderPanic {
        program: &'static str,
        stream: &'static str,
    },
    /// The complete child operation exceeded its admitted duration.
    Timeout {
        program: &'static str,
        duration: Duration,
    },
}

impl ProcessError {
    /// Reports whether the primary process I/O failure is executable absence.
    pub(crate) fn is_not_found(&self) -> bool {
        match self {
            Self::Additional { primary, .. } | Self::Cleanup { primary, .. } => {
                primary.is_not_found()
            }
            Self::Io { source, .. } => source.kind() == io::ErrorKind::NotFound,
            Self::Interrupted { .. }
            | Self::MissingStream { .. }
            | Self::OutputLimit { .. }
            | Self::ReaderPanic { .. }
            | Self::Timeout { .. } => false,
        }
    }
}

impl fmt::Debug for ProcessError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, formatter)
    }
}

impl fmt::Display for ProcessError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Additional {
                primary,
                additional,
            } => write!(formatter, "{primary}; additionally {additional}"),
            Self::Cleanup {
                primary, action, ..
            } => write!(formatter, "{primary}; additionally failed to {action}"),
            Self::Io {
                program, action, ..
            } => write!(formatter, "cannot {action} {program} process"),
            Self::Interrupted { program, signal } => {
                write!(formatter, "{program} process was interrupted by {signal}")
            }
            Self::MissingStream { program, stream } => {
                write!(formatter, "{program} {stream} pipe is unavailable")
            }
            Self::OutputLimit {
                program,
                stream,
                maximum,
            } => write!(
                formatter,
                "{program} {stream} exceeds the {maximum}-byte bound"
            ),
            Self::ReaderPanic { program, stream } => {
                write!(formatter, "{program} {stream} reader panicked")
            }
            Self::Timeout { program, duration } => write!(
                formatter,
                "{program} process exceeded its {}-second deadline",
                duration.as_secs()
            ),
        }
    }
}

impl Error for ProcessError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Additional { primary, .. } => Some(primary),
            Self::Cleanup { source, .. } | Self::Io { source, .. } => Some(source),
            Self::Interrupted { .. }
            | Self::MissingStream { .. }
            | Self::OutputLimit { .. }
            | Self::ReaderPanic { .. }
            | Self::Timeout { .. } => None,
        }
    }
}
