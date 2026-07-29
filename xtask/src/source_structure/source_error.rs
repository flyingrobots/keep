//! This module owns typed source-structure failures and stable diagnostics.

use std::error::Error;
use std::fmt;
use std::io;
use std::path::PathBuf;
use std::string::FromUtf8Error;

use crate::diagnostic::{escaped_controls, escaped_path};
use crate::git_inventory::GitInventoryError;

pub(crate) enum SourceStructureError {
    /// Git's tracked file mode disagrees with the opened worktree object.
    ExecutionModeChanged {
        /// Ambient display path for the disagreed repository entry.
        path: PathBuf,
        /// Canonical tracked mode label admitted from the index.
        tracked: &'static str,
        /// Canonical mode label observed from the opened worktree object.
        worktree: &'static str,
    },
    GitInventory(GitInventoryError),
    /// `git ls-files --stage` returned a malformed, unmerged, or duplicate record.
    GitIndexRecord,
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
    PythonSource(PathBuf),
    RepositoryRootChanged(PathBuf),
    SourceFileChanged(PathBuf),
    Violations {
        maximum: u64,
        paths: Vec<PathBuf>,
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
            Self::ExecutionModeChanged {
                path,
                tracked,
                worktree,
            } => {
                formatter.write_str("repository execution mode differs from the index for `")?;
                escaped_path(formatter, path)?;
                write!(
                    formatter,
                    "`: tracked mode is {tracked}, worktree mode is {worktree}"
                )
            }
            Self::GitInventory(error) => write!(formatter, "{error}"),
            Self::GitIndexRecord => {
                formatter.write_str("git returned an invalid tracked file-mode record")
            }
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
                escaped_path(formatter, path)?;
                formatter.write_str("`")
            }
            Self::RepositoryRootChanged(path) => {
                formatter.write_str("repository root changed during source inspection: `")?;
                escaped_path(formatter, path)?;
                formatter.write_str("`")
            }
            Self::SourceFileChanged(path) => {
                formatter.write_str("repository source changed during inspection: `")?;
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
            Self::ExecutionModeChanged { .. }
            | Self::GitIndexRecord
            | Self::InvalidPath(_)
            | Self::NonRegular(_)
            | Self::PythonSource(_)
            | Self::RepositoryRootChanged(_)
            | Self::SourceFileChanged(_)
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
    paths: &[PathBuf],
) -> fmt::Result {
    write!(
        formatter,
        "repository source modules exceed the {maximum}-line hard maximum"
    )?;
    for path in paths {
        formatter.write_str("; ")?;
        escaped_path(formatter, path)?;
        write!(formatter, ": >{maximum}")?;
    }
    Ok(())
}
