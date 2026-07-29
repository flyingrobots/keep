//! This module owns typed documentation-integrity failures.

mod display;

use std::error::Error;
use std::fmt;
use std::io;
use std::path::PathBuf;
use std::string::FromUtf8Error;

use crate::bounded_process::ProcessError;
use crate::git_inventory::GitInventoryError;

pub(crate) enum DocumentationError {
    CheckFailures {
        first: Box<Self>,
        second: Box<Self>,
    },
    CorpusFileTooLarge {
        corpus: &'static str,
        path: String,
        maximum: u64,
        observed: u64,
    },
    CorpusSizeOverflow(&'static str),
    CorpusTooLarge {
        corpus: &'static str,
        maximum: u64,
        observed: u64,
    },
    CorpusChanged {
        corpus: &'static str,
        path: String,
    },
    EmptyCorpus(&'static str),
    EnvironmentUnavailable(&'static str),
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
    Process(ProcessError),
    /// A filesystem operation could not construct or remove a refusal fixture.
    RefusalFixture {
        action: &'static str,
        source: io::Error,
    },
    /// A malformed-input scenario did not produce its exact reviewed refusal.
    RefusalMismatch {
        scenario: &'static str,
        observed: Option<Box<Self>>,
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
    RepositoryYaml {
        path: &'static str,
        source: yaml_rust2::ScanError,
    },
    RepositoryRootChanged(PathBuf),
    RepositoryRootInspect {
        path: PathBuf,
        source: io::Error,
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
    ToolFailed {
        program: &'static str,
        code: Option<i32>,
        stdout: String,
        stderr: String,
    },
    ToolOutputEncoding {
        program: &'static str,
        stream: &'static str,
        source: FromUtf8Error,
    },
    ToolUnavailable {
        program: &'static str,
        install_version: &'static str,
        source: ProcessError,
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
            Self::CheckFailures { first, .. } => Some(first),
            Self::GitInventory(error) => Some(error),
            Self::Process(error) => Some(error),
            Self::Inspect { source, .. }
            | Self::RefusalFixture { source, .. }
            | Self::RepositoryFileInspect { source, .. }
            | Self::RepositoryRootInspect { source, .. } => Some(source),
            Self::PathEncoding { source, .. } | Self::RepositoryFileEncoding { source, .. } => {
                Some(source)
            }
            Self::RefusalMismatch {
                observed: Some(error),
                ..
            } => Some(error),
            Self::RepositoryJson { source, .. } => Some(source),
            Self::RepositoryYaml { source, .. } => Some(source),
            Self::ToolOutputEncoding { source, .. } => Some(source),
            Self::ToolUnavailable { source, .. } => Some(source),
            Self::CorpusFileTooLarge { .. }
            | Self::CorpusSizeOverflow(_)
            | Self::CorpusTooLarge { .. }
            | Self::CorpusChanged { .. }
            | Self::EmptyCorpus(_)
            | Self::EnvironmentUnavailable(_)
            | Self::InvalidPath { .. }
            | Self::NonRegular { .. }
            | Self::RefusalMismatch { observed: None, .. }
            | Self::RepositoryFileNonRegular(_)
            | Self::RepositoryFileTooLarge { .. }
            | Self::RepositoryContract { .. }
            | Self::RepositoryContractAt { .. }
            | Self::RepositoryRootChanged(_)
            | Self::RepositoryValue { .. }
            | Self::ToolFailed { .. }
            | Self::VersionMismatch { .. } => None,
        }
    }
}

impl From<GitInventoryError> for DocumentationError {
    fn from(error: GitInventoryError) -> Self {
        Self::GitInventory(error)
    }
}
