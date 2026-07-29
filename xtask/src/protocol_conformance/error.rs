//! This module owns typed protocol-conformance failures and source chaining.

use std::error::Error;
use std::fmt;
use std::io;
use std::num::ParseIntError;
use std::path::PathBuf;
use std::string::FromUtf8Error;

use crate::diagnostic::{escaped_controls, escaped_path};
use crate::external_digest::ExternalDigestError;
use xtask::protocol_admission::RelativePathError;

pub(crate) enum ConformanceError {
    ExternalDigest {
        source: ExternalDigestError,
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
    Utf8 {
        path: PathBuf,
        source: FromUtf8Error,
    },
    Violation(String),
}

impl ConformanceError {
    pub(super) const fn external_digest(source: ExternalDigestError) -> Self {
        Self::ExternalDigest { source }
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
            Self::ExternalDigest { source } => fmt::Display::fmt(source, formatter),
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
            Self::ExternalDigest { source } => Some(source),
            Self::Io { source, .. } => Some(source),
            Self::Integer { source, .. } => Some(source),
            Self::Path { source, .. } => Some(source),
            Self::Utf8 { source, .. } => Some(source),
            Self::Violation(_) => None,
        }
    }
}
