//! This module owns typed source-structure failures and stable diagnostics.

use std::error::Error;
use std::fmt;
use std::io;
use std::path::PathBuf;
use std::string::FromUtf8Error;

use crate::diagnostic::{escaped_controls, escaped_path};
use crate::git_inventory::GitInventoryError;

pub(crate) enum SourceStructureError {
    GitInventory(GitInventoryError),
    GitPathEncoding {
        operation: &'static str,
        source: FromUtf8Error,
    },
    Inspect {
        path: PathBuf,
        source: io::Error,
    },
    InvalidPath(String),
    NonRegular(PathBuf),
    PythonSource(String),
    RepositoryRootChanged(PathBuf),
    Violations {
        maximum: u64,
        paths: Vec<String>,
    },
}

impl fmt::Debug for SourceStructureError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, formatter)
    }
}

impl fmt::Display for SourceStructureError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GitInventory(error) => write!(formatter, "{error}"),
            Self::GitPathEncoding { operation, .. } => {
                write!(formatter, "`{operation}` returned a non-UTF-8 path")
            }
            Self::Inspect { path, .. } => {
                formatter.write_str("cannot inspect `")?;
                escaped_path(formatter, path)?;
                formatter.write_str("`")
            }
            Self::InvalidPath(path) => {
                formatter.write_str("git returned unsafe path `")?;
                escaped_controls(formatter, path)?;
                formatter.write_str("`")
            }
            Self::NonRegular(path) => {
                formatter.write_str("repository source module is not a regular file: `")?;
                escaped_path(formatter, path)?;
                formatter.write_str("`")
            }
            Self::PythonSource(path) => {
                formatter.write_str("pure Rust source boundary refuses Python module `")?;
                escaped_controls(formatter, path)?;
                formatter.write_str("`")
            }
            Self::RepositoryRootChanged(path) => {
                formatter.write_str("repository root changed during source inspection: `")?;
                escaped_path(formatter, path)?;
                formatter.write_str("`")
            }
            Self::Violations { maximum, paths } => violations_display(formatter, *maximum, paths),
        }
    }
}

impl Error for SourceStructureError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::GitInventory(error) => Some(error),
            Self::GitPathEncoding { source, .. } => Some(source),
            Self::Inspect { source, .. } => Some(source),
            Self::InvalidPath(_)
            | Self::NonRegular(_)
            | Self::PythonSource(_)
            | Self::RepositoryRootChanged(_)
            | Self::Violations { .. } => None,
        }
    }
}

impl From<GitInventoryError> for SourceStructureError {
    fn from(error: GitInventoryError) -> Self {
        Self::GitInventory(error)
    }
}

fn violations_display(
    formatter: &mut fmt::Formatter<'_>,
    maximum: u64,
    paths: &[String],
) -> fmt::Result {
    write!(
        formatter,
        "repository source modules exceed the {maximum}-line hard maximum"
    )?;
    for path in paths {
        formatter.write_str("; ")?;
        escaped_controls(formatter, path)?;
        write!(formatter, ": >{maximum}")?;
    }
    Ok(())
}
