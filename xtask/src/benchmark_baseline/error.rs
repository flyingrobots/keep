//! Typed failures at the optimized benchmark execution boundary.

use std::error::Error;
use std::fmt;
use std::io;
use std::path::PathBuf;
use std::string::FromUtf8Error;

use crate::diagnostic::{escaped_controls, escaped_path};

pub(crate) enum BenchmarkBaselineError {
    Io {
        action: &'static str,
        target: PathBuf,
        source: io::Error,
    },
    ProcessIo {
        program: &'static str,
        action: &'static str,
        source: io::Error,
    },
    MissingPipe {
        program: &'static str,
        stream: &'static str,
    },
    ReaderThread {
        program: &'static str,
        stream: &'static str,
    },
    OutputBound {
        program: &'static str,
        stream: &'static str,
        maximum: usize,
    },
    ProcessFailed {
        program: &'static str,
        code: Option<i32>,
        stderr: String,
    },
    DiagnosticEncoding {
        program: &'static str,
        source: FromUtf8Error,
    },
    ValueEncoding {
        coordinate: &'static str,
        source: FromUtf8Error,
    },
    InvalidValue {
        coordinate: &'static str,
    },
    AmbientBuildSetting {
        setting: String,
    },
    ExternalCargoConfiguration {
        path: PathBuf,
    },
    ReportViolation {
        reason: &'static str,
    },
}

impl fmt::Debug for BenchmarkBaselineError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, formatter)
    }
}

impl fmt::Display for BenchmarkBaselineError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { action, target, .. } => {
                write!(formatter, "could not {action} `")?;
                escaped_path(formatter, target)?;
                formatter.write_str("`")
            }
            Self::ProcessIo {
                program, action, ..
            } => write!(formatter, "could not {action} `{program}`"),
            Self::MissingPipe { program, stream } => {
                write!(formatter, "`{program}` has no {stream} pipe")
            }
            Self::ReaderThread { program, stream } => {
                write!(formatter, "`{program}` {stream} reader failed")
            }
            Self::OutputBound {
                program,
                stream,
                maximum,
            } => write!(
                formatter,
                "`{program}` exceeded {maximum} admitted {stream} bytes"
            ),
            Self::ProcessFailed {
                program,
                code,
                stderr,
            } => {
                write!(formatter, "`{program}` failed with status {code:?}: ")?;
                escaped_controls(formatter, stderr)
            }
            Self::DiagnosticEncoding { program, .. } => {
                write!(formatter, "`{program}` diagnostics are not UTF-8")
            }
            Self::ValueEncoding { coordinate, .. } => {
                write!(formatter, "`{coordinate}` output is not UTF-8")
            }
            Self::InvalidValue { coordinate } => {
                write!(formatter, "`{coordinate}` output is invalid")
            }
            Self::AmbientBuildSetting { setting } => {
                write!(formatter, "ambient build setting `")?;
                escaped_controls(formatter, setting)?;
                write!(formatter, "` makes benchmark evidence incomparable")
            }
            Self::ExternalCargoConfiguration { path } => {
                write!(formatter, "external Cargo configuration `")?;
                escaped_path(formatter, path)?;
                write!(formatter, "` makes benchmark evidence incomparable")
            }
            Self::ReportViolation { reason } => {
                write!(formatter, "benchmark report violates `{reason}`")
            }
        }
    }
}

impl Error for BenchmarkBaselineError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io { source, .. } | Self::ProcessIo { source, .. } => Some(source),
            Self::DiagnosticEncoding { source, .. } | Self::ValueEncoding { source, .. } => {
                Some(source)
            }
            Self::MissingPipe { .. }
            | Self::ReaderThread { .. }
            | Self::OutputBound { .. }
            | Self::ProcessFailed { .. }
            | Self::InvalidValue { .. }
            | Self::AmbientBuildSetting { .. }
            | Self::ExternalCargoConfiguration { .. }
            | Self::ReportViolation { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io;
    use std::path::PathBuf;

    use super::BenchmarkBaselineError;

    #[test]
    fn io_target_paths_cannot_forge_diagnostic_lines() {
        let error = BenchmarkBaselineError::Io {
            action: "publish report",
            target: PathBuf::from("first\nError: forged\rrewrite\u{1b}[31m"),
            source: io::Error::other("fixture"),
        };

        assert_eq!(
            error.to_string(),
            "could not publish report `first\\nError: forged\\rrewrite\\u{1b}[31m`"
        );
        assert_eq!(error.to_string().lines().count(), 1);
    }
}
