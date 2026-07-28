//! This module owns typed bounded-process failures.

use std::error::Error;
use std::fmt;
use std::io;
use std::time::Duration;

pub(crate) enum ProcessError {
    Cleanup {
        primary: Box<Self>,
        action: &'static str,
        source: io::Error,
    },
    Io {
        program: &'static str,
        action: &'static str,
        source: io::Error,
    },
    MissingStream {
        program: &'static str,
        stream: &'static str,
    },
    OutputLimit {
        program: &'static str,
        stream: &'static str,
        maximum: usize,
    },
    ReaderPanic {
        program: &'static str,
        stream: &'static str,
    },
    Timeout {
        program: &'static str,
        duration: Duration,
    },
}

impl ProcessError {
    pub(crate) fn is_not_found(&self) -> bool {
        match self {
            Self::Cleanup { primary, .. } => primary.is_not_found(),
            Self::Io { source, .. } => source.kind() == io::ErrorKind::NotFound,
            Self::MissingStream { .. }
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
            Self::Cleanup {
                primary, action, ..
            } => write!(
                formatter,
                "{primary}; additionally failed to {action} child process"
            ),
            Self::Io {
                program, action, ..
            } => write!(formatter, "cannot {action} {program} process"),
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
            Self::Cleanup { source, .. } | Self::Io { source, .. } => Some(source),
            Self::MissingStream { .. }
            | Self::OutputLimit { .. }
            | Self::ReaderPanic { .. }
            | Self::Timeout { .. } => None,
        }
    }
}
