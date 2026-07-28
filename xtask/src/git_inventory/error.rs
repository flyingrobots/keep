//! This module owns typed Git inventory failures and stable diagnostics.

use std::error::Error;
use std::fmt::{self, Write as _};
use std::io;
use std::string::FromUtf8Error;

use crate::diagnostic::escaped_controls;

#[derive(Clone, Copy)]
/// The unit named by a bounded Git-output failure.
pub(crate) enum GitOutputUnit {
    /// A byte-count bound.
    Bytes,
    /// A path-record count bound.
    Items,
}

/// A typed failure while listing or decoding repository paths from Git.
pub(crate) enum GitInventoryError {
    /// Cleanup failed after an earlier inventory failure was already detected.
    Cleanup {
        primary: Box<Self>,
        cleanup: Box<Self>,
    },
    /// Git emitted the same path record more than once.
    DuplicatePath(Vec<u8>),
    /// Git emitted an empty NUL-framed path record.
    EmptyPath { operation: &'static str },
    /// Git exited unsuccessfully and returned valid UTF-8 diagnostics.
    Failed {
        operation: &'static str,
        code: Option<i32>,
        stderr: String,
    },
    /// Git exited unsuccessfully with diagnostics that were not UTF-8.
    DiagnosticEncoding {
        operation: &'static str,
        code: Option<i32>,
        source: FromUtf8Error,
    },
    /// A retained byte or item count exceeded its fixed bound.
    OutputBound {
        operation: &'static str,
        stream: &'static str,
        maximum: usize,
        unit: GitOutputUnit,
    },
    /// Git ended its output with bytes not terminated by a NUL delimiter.
    OutputFraming { operation: &'static str },
    /// A Git child configured for capture did not expose a requested pipe.
    Pipe {
        operation: &'static str,
        stream: &'static str,
    },
    /// A named operating-system action for the Git child failed.
    Run {
        operation: &'static str,
        action: &'static str,
        source: io::Error,
    },
    /// The concurrent diagnostic-reader thread stopped by panicking.
    Worker { operation: &'static str },
}

impl fmt::Debug for GitInventoryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, formatter)
    }
}

impl fmt::Display for GitInventoryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cleanup { primary, cleanup } => {
                write!(
                    formatter,
                    "{primary}; additionally, cleanup failed: {cleanup}"
                )
            }
            Self::DuplicatePath(path) => {
                formatter.write_str("git returned duplicate path `")?;
                escaped_bytes(formatter, path)?;
                formatter.write_str("`")
            }
            Self::EmptyPath { operation } => {
                write!(formatter, "`{operation}` returned an empty path")
            }
            Self::Failed {
                operation,
                code,
                stderr,
            } => git_failed(formatter, operation, *code, stderr),
            Self::DiagnosticEncoding {
                operation, code, ..
            } => write!(
                formatter,
                "`{operation}` failed with code {code:?} and returned non-UTF-8 diagnostics"
            ),
            Self::OutputBound {
                operation,
                stream,
                maximum,
                unit,
            } => write!(
                formatter,
                "`{operation}` exceeded the {stream} bound of {maximum} {}",
                unit.label()
            ),
            Self::OutputFraming { operation } => {
                write!(
                    formatter,
                    "`{operation}` returned a non-NUL-terminated path"
                )
            }
            Self::Pipe { operation, stream } => {
                write!(formatter, "`{operation}` did not provide its {stream} pipe")
            }
            Self::Run {
                operation, action, ..
            } => write!(formatter, "cannot {action} `{operation}`"),
            Self::Worker { operation } => {
                write!(
                    formatter,
                    "`{operation}` diagnostic reader stopped unexpectedly"
                )
            }
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

impl Error for GitInventoryError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Cleanup { primary, .. } => Some(primary),
            Self::DiagnosticEncoding { source, .. } => Some(source),
            Self::Run { source, .. } => Some(source),
            Self::DuplicatePath(_)
            | Self::EmptyPath { .. }
            | Self::Failed { .. }
            | Self::OutputBound { .. }
            | Self::OutputFraming { .. }
            | Self::Pipe { .. }
            | Self::Worker { .. } => None,
        }
    }
}

fn git_failed(
    formatter: &mut fmt::Formatter<'_>,
    operation: &str,
    code: Option<i32>,
    stderr: &str,
) -> fmt::Result {
    write!(formatter, "`{operation}` failed with code {code:?}")?;
    if stderr.is_empty() {
        return Ok(());
    }
    formatter.write_str(": ")?;
    escaped_controls(formatter, stderr.trim_end())
}

fn escaped_bytes(formatter: &mut fmt::Formatter<'_>, bytes: &[u8]) -> fmt::Result {
    for byte in bytes {
        if byte.is_ascii_graphic() || *byte == b' ' {
            formatter.write_char(char::from(*byte))?;
        } else {
            write!(formatter, "\\x{byte:02x}")?;
        }
    }
    Ok(())
}
