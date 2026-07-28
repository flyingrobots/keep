//! This module owns typed protocol-conformance failures and source chaining.

use std::error::Error;
use std::fmt;
use std::io;
use std::num::ParseIntError;
use std::path::PathBuf;
use std::string::FromUtf8Error;
use std::time::Duration;

use crate::diagnostic::{escaped_controls, escaped_path};
use xtask::protocol_admission::RelativePathError;

pub(crate) enum ConformanceError {
    Cleanup {
        primary: Box<Self>,
        action: &'static str,
        source: io::Error,
    },
    Integer {
        field: String,
        source: ParseIntError,
    },
    Io {
        action: &'static str,
        path: PathBuf,
        source: io::Error,
    },
    Path {
        parameter: String,
        source: RelativePathError,
    },
    ProcessDiagnosticEncoding {
        program: &'static str,
        code: Option<i32>,
        source: FromUtf8Error,
    },
    ProcessFailed {
        program: &'static str,
        code: Option<i32>,
        stderr: String,
    },
    ProcessOutputBound {
        program: &'static str,
        stream: &'static str,
        maximum: usize,
    },
    ProcessTimeout {
        program: &'static str,
        duration: Duration,
    },
    ReaderPanic {
        program: &'static str,
        stream: &'static str,
    },
    WriterPanic {
        program: &'static str,
    },
    Utf8 {
        path: PathBuf,
        source: FromUtf8Error,
    },
    Violation(String),
}

impl ConformanceError {
    pub(super) fn cleanup(primary: Self, action: &'static str, source: io::Error) -> Self {
        Self::Cleanup {
            primary: Box::new(primary),
            action,
            source,
        }
    }

    pub(super) fn io(action: &'static str, path: impl Into<PathBuf>, source: io::Error) -> Self {
        Self::Io {
            action,
            path: path.into(),
            source,
        }
    }

    pub(super) fn violation(message: impl Into<String>) -> Self {
        Self::Violation(message.into())
    }
}

impl fmt::Debug for ConformanceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, formatter)
    }
}

impl ConformanceError {
    fn fmt_body(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cleanup {
                primary,
                action,
                source,
            } => {
                primary.fmt_body(formatter)?;
                write!(formatter, "; cleanup could not {action}: {source}")
            }
            Self::Integer { field, .. } => {
                formatter.write_str("cannot parse canonical ")?;
                escaped_controls(formatter, field)
            }
            Self::Io { action, path, .. } => {
                write!(formatter, "cannot {action} `")?;
                escaped_path(formatter, path)?;
                formatter.write_str("`")
            }
            Self::Path { parameter, source } => {
                formatter.write_str("unsafe corpus path: ")?;
                escaped_controls(formatter, parameter)?;
                write!(formatter, ": {source}")
            }
            Self::ProcessDiagnosticEncoding { program, code, .. } => write!(
                formatter,
                "{program} failed with status {code:?} and non-UTF-8 diagnostics"
            ),
            Self::ProcessFailed {
                program,
                code,
                stderr,
            } => {
                write!(formatter, "{program} failed with status {code:?}: ")?;
                escaped_controls(formatter, stderr)
            }
            Self::ProcessOutputBound {
                program,
                stream,
                maximum,
            } => write!(
                formatter,
                "{program} {stream} exceeded the {maximum}-byte bound"
            ),
            Self::ProcessTimeout { program, duration } => {
                write!(formatter, "{program} exceeded {duration:?}")
            }
            Self::ReaderPanic { program, stream } => {
                write!(formatter, "{program} {stream} reader panicked")
            }
            Self::WriterPanic { program } => write!(formatter, "{program} stdin writer panicked"),
            Self::Utf8 { path, .. } => {
                escaped_path(formatter, path)?;
                formatter.write_str(": protocol is not UTF-8")
            }
            Self::Violation(message) => escaped_controls(formatter, message),
        }
    }
}

impl fmt::Display for ConformanceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("protocol conformance check failed: ")?;
        self.fmt_body(formatter)
    }
}

impl Error for ConformanceError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Cleanup { primary, .. } => Some(primary.as_ref()),
            Self::Io { source, .. } => Some(source),
            Self::Integer { source, .. } => Some(source),
            Self::Path { source, .. } => Some(source),
            Self::ProcessDiagnosticEncoding { source, .. } | Self::Utf8 { source, .. } => {
                Some(source)
            }
            Self::ProcessFailed { .. }
            | Self::ProcessOutputBound { .. }
            | Self::ProcessTimeout { .. }
            | Self::ReaderPanic { .. }
            | Self::WriterPanic { .. }
            | Self::Violation(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::io;

    use super::ConformanceError;

    #[test]
    fn cleanup_preserves_one_prefix_and_the_primary_source() {
        let error = ConformanceError::cleanup(
            ConformanceError::violation("primary refusal"),
            "close witness",
            io::Error::other("cleanup refusal"),
        );
        assert_eq!(
            error.to_string(),
            "protocol conformance check failed: primary refusal; \
             cleanup could not close witness: cleanup refusal"
        );
        assert!(matches!(
            error.source(),
            Some(source)
                if source.to_string()
                    == "protocol conformance check failed: primary refusal"
        ));
    }
}
