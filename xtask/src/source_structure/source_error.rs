//! This module owns typed source-structure failures and stable diagnostics.

use std::error::Error;
use std::fmt::{self, Write as _};
use std::io;
use std::path::PathBuf;
use std::string::FromUtf8Error;

#[derive(Clone, Copy)]
pub(crate) enum GitOutputUnit {
    Bytes,
    Items,
}

pub(crate) enum SourceStructureError {
    DuplicatePath(String),
    GitFailed {
        operation: &'static str,
        code: Option<i32>,
        stderr: String,
    },
    GitDiagnosticEncoding {
        operation: &'static str,
        code: Option<i32>,
        source: FromUtf8Error,
    },
    GitPathEncoding {
        operation: &'static str,
        source: FromUtf8Error,
    },
    GitOutputBound {
        operation: &'static str,
        stream: &'static str,
        maximum: usize,
        unit: GitOutputUnit,
    },
    GitOutputFraming {
        operation: &'static str,
    },
    GitPipe {
        operation: &'static str,
        stream: &'static str,
    },
    GitWorker {
        operation: &'static str,
    },
    Inspect {
        path: PathBuf,
        source: io::Error,
    },
    InvalidPath(String),
    NonRegular(PathBuf),
    RunGit {
        operation: &'static str,
        action: &'static str,
        source: io::Error,
    },
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
            Self::DuplicatePath(path) => {
                write!(formatter, "git returned duplicate path `{path}`")
            }
            Self::GitFailed {
                operation,
                code,
                stderr,
            } => git_failed(formatter, operation, *code, stderr),
            Self::GitDiagnosticEncoding {
                operation, code, ..
            } => write!(
                formatter,
                "`{operation}` failed with code {code:?} and returned non-UTF-8 diagnostics"
            ),
            Self::GitPathEncoding { operation, .. } => {
                write!(formatter, "`{operation}` returned a non-UTF-8 path")
            }
            Self::GitOutputBound {
                operation,
                stream,
                maximum,
                unit,
            } => write!(
                formatter,
                "`{operation}` exceeded the {stream} bound of {maximum} {}",
                unit.label()
            ),
            Self::GitOutputFraming { operation } => {
                write!(
                    formatter,
                    "`{operation}` returned a non-NUL-terminated path"
                )
            }
            Self::GitPipe { operation, stream } => {
                write!(formatter, "`{operation}` did not provide its {stream} pipe")
            }
            Self::GitWorker { operation } => {
                write!(
                    formatter,
                    "`{operation}` diagnostic reader stopped unexpectedly"
                )
            }
            Self::Inspect { path, .. } => {
                write!(formatter, "cannot inspect `{}`", path.display())
            }
            Self::InvalidPath(path) => write!(formatter, "git returned unsafe path `{path}`"),
            Self::NonRegular(path) => write!(
                formatter,
                "tracked source module is not a regular file: `{}`",
                path.display()
            ),
            Self::RunGit {
                operation, action, ..
            } => write!(formatter, "cannot {action} `{operation}`"),
            Self::Violations { maximum, paths } => violations_display(formatter, *maximum, paths),
        }
    }
}

impl GitOutputUnit {
    const fn label(self) -> &'static str {
        match self {
            Self::Bytes => "bytes",
            Self::Items => "items",
        }
    }
}

impl Error for SourceStructureError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::GitDiagnosticEncoding { source, .. } | Self::GitPathEncoding { source, .. } => {
                Some(source)
            }
            Self::Inspect { source, .. } | Self::RunGit { source, .. } => Some(source),
            Self::DuplicatePath(_)
            | Self::GitFailed { .. }
            | Self::GitOutputBound { .. }
            | Self::GitOutputFraming { .. }
            | Self::GitPipe { .. }
            | Self::GitWorker { .. }
            | Self::InvalidPath(_)
            | Self::NonRegular(_)
            | Self::Violations { .. } => None,
        }
    }
}

fn git_failed(
    formatter: &mut fmt::Formatter<'_>,
    operation: &str,
    code: Option<i32>,
    stderr: &str,
) -> fmt::Result {
    write!(formatter, "`{operation}` failed with code {code:?}: ")?;
    escaped_controls(formatter, stderr.trim())
}

fn escaped_controls(formatter: &mut fmt::Formatter<'_>, diagnostic: &str) -> fmt::Result {
    for character in diagnostic.chars() {
        if character.is_control() {
            for escaped in character.escape_default() {
                formatter.write_char(escaped)?;
            }
        } else {
            formatter.write_char(character)?;
        }
    }
    Ok(())
}

fn violations_display(
    formatter: &mut fmt::Formatter<'_>,
    maximum: u64,
    paths: &[String],
) -> fmt::Result {
    write!(
        formatter,
        "tracked source modules exceed the {maximum}-line hard maximum"
    )?;
    for path in paths {
        write!(formatter, "; {path}: >{maximum}")?;
    }
    Ok(())
}
