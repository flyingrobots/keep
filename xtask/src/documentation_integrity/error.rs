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
    RepositoryFileEncoding {
        path: &'static str,
        source: FromUtf8Error,
    },
    RepositoryFileInspect {
        path: &'static str,
        source: io::Error,
    },
    RepositoryFileNonRegular(&'static str),
    RepositoryFileTooLarge {
        path: &'static str,
        maximum: u64,
    },
    RepositoryContract {
        path: &'static str,
        requirement: &'static str,
    },
    RepositoryContractAt {
        path: &'static str,
        subject: String,
        requirement: &'static str,
    },
    RepositoryJson {
        path: &'static str,
        source: serde_json::Error,
    },
    RepositoryValue {
        path: &'static str,
        field: &'static str,
        expected: &'static str,
        observed: Option<String>,
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
            Self::RepositoryFileEncoding { path, .. } => {
                write!(formatter, "repository file `{path}` is not UTF-8")
            }
            Self::RepositoryFileInspect { path, .. } => {
                write!(formatter, "cannot inspect repository file `{path}`")
            }
            Self::RepositoryFileNonRegular(path) => {
                write!(formatter, "repository file is not regular: `{path}`")
            }
            Self::RepositoryFileTooLarge { path, maximum } => write!(
                formatter,
                "repository file `{path}` exceeds the {maximum}-byte bound"
            ),
            Self::RepositoryContract { path, requirement } => {
                write!(
                    formatter,
                    "repository file `{path}` violates: {requirement}"
                )
            }
            Self::RepositoryContractAt {
                path,
                subject,
                requirement,
            } => {
                write!(
                    formatter,
                    "repository file `{path}` violates {requirement} at `"
                )?;
                escaped_controls(formatter, subject)?;
                formatter.write_str("`")
            }
            Self::RepositoryJson { path, .. } => {
                write!(formatter, "repository file `{path}` is not valid JSON")
            }
            Self::RepositoryValue {
                path,
                field,
                expected,
                observed,
            } => {
                write!(
                    formatter,
                    "repository file `{path}` requires `{field}` to be {expected:?}; observed "
                )?;
                match observed {
                    Some(value) => {
                        formatter.write_str("\"")?;
                        escaped_controls(formatter, value)?;
                        formatter.write_str("\"")
                    }
                    None => formatter.write_str("missing"),
                }
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
            Self::Inspect { source, .. } | Self::RepositoryFileInspect { source, .. } => {
                Some(source)
            }
            Self::PathEncoding { source, .. } | Self::RepositoryFileEncoding { source, .. } => {
                Some(source)
            }
            Self::RepositoryJson { source, .. } => Some(source),
            Self::EmptyCorpus(_)
            | Self::InvalidPath { .. }
            | Self::NonRegular { .. }
            | Self::RepositoryFileNonRegular(_)
            | Self::RepositoryFileTooLarge { .. }
            | Self::RepositoryContract { .. }
            | Self::RepositoryContractAt { .. }
            | Self::RepositoryValue { .. }
            | Self::VersionMismatch { .. } => None,
        }
    }
}

impl From<GitInventoryError> for DocumentationError {
    fn from(error: GitInventoryError) -> Self {
        Self::GitInventory(error)
    }
}
