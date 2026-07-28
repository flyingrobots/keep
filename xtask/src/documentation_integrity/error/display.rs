//! This module owns human-readable documentation-integrity diagnostics.

use std::fmt;

use crate::diagnostic::{escaped_controls, escaped_path};

use super::DocumentationError;

#[derive(Clone, Copy)]
enum SourcePathDiagnostic {
    Inspect,
    Invalid,
    NonRegular,
}

#[derive(Clone, Copy)]
enum RepositoryRootDiagnostic {
    Changed,
    Inspect,
}

impl fmt::Display for DocumentationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CheckFailures { first, second } => {
                write!(formatter, "{first}; additionally: {second}")
            }
            Self::EmptyCorpus(label) => write!(formatter, "the {label} corpus is empty"),
            Self::GitInventory(error) => write!(formatter, "{error}"),
            Self::Inspect { corpus, path, .. } => {
                source_path(formatter, SourcePathDiagnostic::Inspect, corpus, path)
            }
            Self::InvalidPath { corpus, path } => {
                source_path(formatter, SourcePathDiagnostic::Invalid, corpus, path)
            }
            Self::NonRegular { corpus, path } => {
                source_path(formatter, SourcePathDiagnostic::NonRegular, corpus, path)
            }
            Self::PathEncoding { corpus, .. } => {
                write!(formatter, "{corpus} corpus contains a non-UTF-8 path")
            }
            Self::Process(error) => write!(formatter, "{error}"),
            error @ (Self::RepositoryFileEncoding { .. }
            | Self::RepositoryFileInspect { .. }
            | Self::RepositoryFileNonRegular(_)
            | Self::RepositoryFileTooLarge { .. }
            | Self::RepositoryContract { .. }
            | Self::RepositoryContractAt { .. }
            | Self::RepositoryJson { .. }
            | Self::RepositoryYaml { .. }
            | Self::RepositoryValue { .. }) => repository_file(formatter, error),
            Self::RepositoryRootChanged(path) => {
                repository_root(formatter, RepositoryRootDiagnostic::Changed, path)
            }
            Self::RepositoryRootInspect { path, .. } => {
                repository_root(formatter, RepositoryRootDiagnostic::Inspect, path)
            }
            error @ (Self::VersionMismatch { .. }
            | Self::ToolFailed { .. }
            | Self::ToolOutputEncoding { .. }
            | Self::ToolUnavailable { .. }) => tool(formatter, error),
        }
    }
}

fn repository_file(formatter: &mut fmt::Formatter<'_>, error: &DocumentationError) -> fmt::Result {
    match error {
        DocumentationError::RepositoryFileEncoding { path, .. } => {
            write!(formatter, "repository file `{path}` is not UTF-8")
        }
        DocumentationError::RepositoryFileInspect { path, .. } => {
            write!(formatter, "cannot inspect repository file `{path}`")
        }
        DocumentationError::RepositoryFileNonRegular(path) => {
            write!(formatter, "repository file is not regular: `{path}`")
        }
        DocumentationError::RepositoryFileTooLarge { path, maximum } => write!(
            formatter,
            "repository file `{path}` exceeds the {maximum}-byte bound"
        ),
        DocumentationError::RepositoryContract { path, requirement } => {
            write!(
                formatter,
                "repository file `{path}` violates: {requirement}"
            )
        }
        DocumentationError::RepositoryContractAt {
            path,
            subject,
            requirement,
        } => repository_contract_at(formatter, path, subject, requirement),
        DocumentationError::RepositoryJson { path, .. } => {
            write!(formatter, "repository file `{path}` is not valid JSON")
        }
        DocumentationError::RepositoryYaml { path, .. } => {
            write!(formatter, "repository file `{path}` is not valid YAML")
        }
        DocumentationError::RepositoryValue {
            path,
            field,
            expected,
            observed,
        } => repository_value(formatter, path, field, expected, observed.as_deref()),
        _ => Err(fmt::Error),
    }
}

fn tool(formatter: &mut fmt::Formatter<'_>, error: &DocumentationError) -> fmt::Result {
    match error {
        DocumentationError::VersionMismatch {
            program,
            expected,
            observed,
        } => write!(
            formatter,
            "{program} version mismatch: expected {expected:?}, observed {observed:?}"
        ),
        DocumentationError::ToolFailed {
            program,
            code,
            stdout,
            stderr,
        } => tool_failed(formatter, program, *code, stdout, stderr),
        DocumentationError::ToolOutputEncoding {
            program, stream, ..
        } => {
            write!(formatter, "{program} {stream} is not UTF-8")
        }
        DocumentationError::ToolUnavailable {
            program,
            install_version,
            ..
        } => write!(
            formatter,
            "{program} is unavailable; install version {install_version}"
        ),
        _ => Err(fmt::Error),
    }
}

fn source_path(
    formatter: &mut fmt::Formatter<'_>,
    diagnostic: SourcePathDiagnostic,
    corpus: &str,
    path: &str,
) -> fmt::Result {
    match diagnostic {
        SourcePathDiagnostic::Inspect => write!(formatter, "cannot inspect {corpus} source `")?,
        SourcePathDiagnostic::Invalid => {
            write!(formatter, "{corpus} corpus contains an unsafe path `")?;
        }
        SourcePathDiagnostic::NonRegular => {
            write!(formatter, "{corpus} source is not a regular file: `")?;
        }
    }
    escaped_controls(formatter, path)?;
    formatter.write_str("`")
}

fn repository_contract_at(
    formatter: &mut fmt::Formatter<'_>,
    path: &str,
    subject: &str,
    requirement: &str,
) -> fmt::Result {
    write!(
        formatter,
        "repository file `{path}` violates {requirement} at `"
    )?;
    escaped_controls(formatter, subject)?;
    formatter.write_str("`")
}

fn repository_root(
    formatter: &mut fmt::Formatter<'_>,
    diagnostic: RepositoryRootDiagnostic,
    path: &std::path::Path,
) -> fmt::Result {
    match diagnostic {
        RepositoryRootDiagnostic::Changed => {
            formatter.write_str("documentation repository root changed: `")?;
        }
        RepositoryRootDiagnostic::Inspect => {
            formatter.write_str("cannot inspect documentation repository root: `")?;
        }
    }
    escaped_path(formatter, path)?;
    formatter.write_str("`")
}

fn repository_value(
    formatter: &mut fmt::Formatter<'_>,
    path: &str,
    field: &str,
    expected: &str,
    observed: Option<&str>,
) -> fmt::Result {
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

fn tool_failed(
    formatter: &mut fmt::Formatter<'_>,
    program: &str,
    code: Option<i32>,
    stdout: &str,
    stderr: &str,
) -> fmt::Result {
    write!(formatter, "{program} failed with exit status ")?;
    match code {
        Some(code) => write!(formatter, "{code}")?,
        None => formatter.write_str("unavailable")?,
    }
    formatter.write_str("; stdout \"")?;
    escaped_controls(formatter, stdout)?;
    formatter.write_str("\"; stderr \"")?;
    escaped_controls(formatter, stderr)?;
    formatter.write_str("\"")
}
