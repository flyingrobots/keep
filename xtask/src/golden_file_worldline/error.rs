//! This module owns typed Golden File Worldline failures and source chaining.

use std::error::Error;
use std::fmt;
use std::io;
use std::num::ParseIntError;
use std::path::PathBuf;
use std::string::FromUtf8Error;

use crate::diagnostic::{escaped_controls, escaped_path};
use xtask::protocol_admission::RelativePathError;

pub(crate) enum GoldenError {
    ExternalDigest {
        source: Box<dyn Error + Send + Sync>,
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
    Utf8 {
        path: PathBuf,
        source: FromUtf8Error,
    },
    Path {
        parameter: String,
        source: RelativePathError,
    },
    Violation(String),
}

impl GoldenError {
    pub(super) fn external_digest(source: impl Error + Send + Sync + 'static) -> Self {
        Self::ExternalDigest {
            source: Box::new(source),
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

impl fmt::Debug for GoldenError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, formatter)
    }
}

impl fmt::Display for GoldenError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("golden corpus check failed: ")?;
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
            Self::Utf8 { path, .. } => {
                escaped_path(formatter, path)?;
                formatter.write_str(": protocol is not UTF-8")
            }
            Self::Path { parameter, source } => {
                formatter.write_str("unsafe source path: ")?;
                escaped_controls(formatter, parameter)?;
                write!(formatter, ": {source}")
            }
            Self::Violation(message) => escaped_controls(formatter, message),
        }
    }
}

impl Error for GoldenError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ExternalDigest { source } => Some(source.as_ref()),
            Self::Integer { source, .. } => Some(source),
            Self::Io { source, .. } => Some(source),
            Self::Utf8 { source, .. } => Some(source),
            Self::Path { source, .. } => Some(source),
            Self::Violation(_) => None,
        }
    }
}
