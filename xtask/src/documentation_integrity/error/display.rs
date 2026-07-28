//! This module owns human-readable documentation-integrity diagnostics.

use std::fmt;

use crate::diagnostic::escaped_controls;

use super::DocumentationError;

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
