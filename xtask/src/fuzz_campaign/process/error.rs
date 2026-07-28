//! This module owns typed cargo-fuzz process failures.

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
        action: &'static str,
        source: io::Error,
    },
    MissingStream(&'static str),
    OutputLimit {
        stream: &'static str,
        maximum: usize,
    },
    ReaderPanic(&'static str),
    Timeout(Duration),
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
            Self::Io { action, .. } => write!(formatter, "cannot {action} cargo-fuzz process"),
            Self::MissingStream(stream) => {
                write!(formatter, "cargo-fuzz {stream} pipe is unavailable")
            }
            Self::OutputLimit { stream, maximum } => {
                write!(
                    formatter,
                    "cargo-fuzz {stream} exceeds the {maximum}-byte bound"
                )
            }
            Self::ReaderPanic(stream) => write!(formatter, "cargo-fuzz {stream} reader panicked"),
            Self::Timeout(duration) => write!(
                formatter,
                "cargo-fuzz process exceeded its {}-second deadline",
                duration.as_secs()
            ),
        }
    }
}

impl Error for ProcessError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Cleanup { source, .. } | Self::Io { source, .. } => Some(source),
            Self::MissingStream(_)
            | Self::OutputLimit { .. }
            | Self::ReaderPanic(_)
            | Self::Timeout(_) => None,
        }
    }
}
