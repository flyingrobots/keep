//! This module owns typed Golden File Worldline failures and source chaining.

use std::error::Error;
use std::fmt;
use std::io;
use std::num::ParseIntError;
use std::path::PathBuf;
use std::string::FromUtf8Error;

use crate::diagnostic::{escaped_controls, escaped_path};

pub(crate) enum GoldenError {
    Integer {
        field: String,
        source: ParseIntError,
    },
    Io {
        action: &'static str,
        path: PathBuf,
        source: io::Error,
    },
    Utf8 {
        path: PathBuf,
        source: FromUtf8Error,
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
    Violation(String),
}

impl GoldenError {
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

impl fmt::Debug for GoldenError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, formatter)
    }
}

impl fmt::Display for GoldenError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("golden corpus check failed: ")?;
        match self {
            Self::Integer { field, .. } => {
                formatter.write_str("cannot parse canonical ")?;
                escaped_controls(formatter, field)
            }
            Self::Io { action, path, .. } => {
                write!(formatter, "cannot {action} `")?;
                escaped_path(formatter, path)?;
                formatter.write_str("`")
            }
            Self::Utf8 { path, .. } => {
                escaped_path(formatter, path)?;
                formatter.write_str(": protocol is not UTF-8")
            }
            Self::ProcessDiagnosticEncoding { program, code, .. } => {
                write!(
                    formatter,
                    "{program} failed with status {code:?} and non-UTF-8 diagnostics"
                )
            }
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
            Self::Violation(message) => escaped_controls(formatter, message),
        }
    }
}

impl Error for GoldenError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Integer { source, .. } => Some(source),
            Self::Io { source, .. } => Some(source),
            Self::Utf8 { source, .. } | Self::ProcessDiagnosticEncoding { source, .. } => {
                Some(source)
            }
            Self::ProcessFailed { .. } | Self::ProcessOutputBound { .. } | Self::Violation(_) => {
                None
            }
        }
    }
}
