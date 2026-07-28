//! This module owns typed documentation-integrity failures.

use std::error::Error;
use std::fmt;
use std::io;
use std::string::FromUtf8Error;

use crate::diagnostic::escaped_controls;
use crate::git_inventory::GitInventoryError;

pub(super) enum DocumentationError {
    EmptyCorpus(&'static str),
    GitInventory(GitInventoryError),
    Inspect {
        corpus: &'static str,
        path: String,
        source: io::Error,
    },
    InvalidPath {
        corpus: &'static str,
        path: String,
    },
    NonRegular {
        corpus: &'static str,
        path: String,
    },
    PathEncoding {
        corpus: &'static str,
        source: FromUtf8Error,
    },
    VersionMismatch {
        program: &'static str,
        expected: &'static str,
        observed: String,
    },
}

impl fmt::Debug for DocumentationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, formatter)
    }
}

impl fmt::Display for DocumentationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyCorpus(label) => write!(formatter, "the {label} corpus is empty"),
            Self::GitInventory(error) => write!(formatter, "{error}"),
            Self::Inspect { corpus, path, .. } => {
                write!(formatter, "cannot inspect {corpus} source `")?;
                escaped_controls(formatter, path)?;
                formatter.write_str("`")
            }
            Self::InvalidPath { corpus, path } => {
                write!(formatter, "{corpus} corpus contains an unsafe path `")?;
                escaped_controls(formatter, path)?;
                formatter.write_str("`")
            }
            Self::NonRegular { corpus, path } => {
                write!(formatter, "{corpus} source is not a regular file: `")?;
                escaped_controls(formatter, path)?;
                formatter.write_str("`")
            }
            Self::PathEncoding { corpus, .. } => {
                write!(formatter, "{corpus} corpus contains a non-UTF-8 path")
            }
            Self::VersionMismatch {
                program,
                expected,
                observed,
            } => write!(
                formatter,
                "{program} version mismatch: expected {expected:?}, observed {observed:?}"
            ),
        }
    }
}

impl Error for DocumentationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::GitInventory(error) => Some(error),
            Self::Inspect { source, .. } => Some(source),
            Self::PathEncoding { source, .. } => Some(source),
            Self::EmptyCorpus(_)
            | Self::InvalidPath { .. }
            | Self::NonRegular { .. }
            | Self::VersionMismatch { .. } => None,
        }
    }
}

impl From<GitInventoryError> for DocumentationError {
    fn from(error: GitInventoryError) -> Self {
        Self::GitInventory(error)
    }
}
