//! This module owns typed documentation-integrity failures.

mod display;

use std::error::Error;
use std::fmt;
use std::io;
use std::string::FromUtf8Error;

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
