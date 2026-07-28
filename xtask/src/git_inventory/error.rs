//! This module owns typed Git inventory failures and stable diagnostics.

use std::error::Error;
use std::fmt::{self, Write as _};
use std::io;
use std::string::FromUtf8Error;

use crate::diagnostic::escaped_controls;

#[derive(Clone, Copy)]
pub(crate) enum GitOutputUnit {
    Bytes,
    Items,
}

pub(crate) enum GitInventoryError {
    Cleanup {
        primary: Box<Self>,
        cleanup: Box<Self>,
    },
    DuplicatePath(Vec<u8>),
    Failed {
        operation: &'static str,
        code: Option<i32>,
        stderr: String,
    },
    DiagnosticEncoding {
        operation: &'static str,
        code: Option<i32>,
        source: FromUtf8Error,
    },
    OutputBound {
        operation: &'static str,
        stream: &'static str,
        maximum: usize,
        unit: GitOutputUnit,
    },
    OutputFraming {
        operation: &'static str,
    },
    Pipe {
        operation: &'static str,
        stream: &'static str,
    },
    Run {
        operation: &'static str,
        action: &'static str,
        source: io::Error,
    },
    Worker {
        operation: &'static str,
    },
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
