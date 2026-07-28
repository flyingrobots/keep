//! This module owns exact fuzz target discovery failures.

use std::error::Error;
use std::fmt;
use std::io;
use std::path::PathBuf;

use crate::bounded_process::ProcessError;
use crate::diagnostic::{escaped_controls, escaped_path};

pub(crate) enum TargetError {
    Disagreement {
        expected: Vec<String>,
        observed: Vec<String>,
    },
    Duplicate,
    EmptyHarnesses,
    EmptyRegistry,
    Inspect {
        path: PathBuf,
        source: io::Error,
    },
    InvalidEncoding,
    ListFailed {
        stdout: Vec<u8>,
        stderr: Vec<u8>,
    },
    Malformed(String),
    MalformedPath(PathBuf),
    NonRegular(PathBuf),
    Process(ProcessError),
    ReadDirectory {
        path: PathBuf,
        source: io::Error,
    },
    ReadEntry {
        source: io::Error,
    },
}

impl fmt::Debug for TargetError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, formatter)
    }
}

impl fmt::Display for TargetError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Disagreement { expected, observed } => write!(
                formatter,
                "registered fuzz targets differ from harnesses: expected {}, observed {}",
                Names(expected),
                Names(observed)
            ),
            Self::Duplicate => formatter.write_str("cargo fuzz list returned duplicate targets"),
            Self::EmptyHarnesses => formatter.write_str("checked-in fuzz target set is empty"),
            Self::EmptyRegistry => formatter.write_str("cargo fuzz list returned no targets"),
            Self::Inspect { path, .. } => {
                formatter.write_str("cannot inspect fuzz harness ")?;
                escaped_path(formatter, path)
            }
            Self::InvalidEncoding => {
                formatter.write_str("cargo fuzz list output is not valid UTF-8")
            }
            Self::ListFailed { stdout, stderr } => write!(
                formatter,
                "cargo fuzz list failed with stdout {stdout:?} and stderr {stderr:?}"
            ),
            Self::Malformed(name) => {
                formatter.write_str("malformed fuzz target `")?;
                escaped_controls(formatter, name)?;
                formatter.write_str("`")
            }
            Self::MalformedPath(path) => {
                formatter.write_str("fuzz harness path is not valid Unicode: ")?;
                escaped_path(formatter, path)
            }
            Self::NonRegular(path) => {
                formatter.write_str("fuzz harness is not a regular file: ")?;
                escaped_path(formatter, path)
            }
            Self::Process(error) => write!(formatter, "{error}"),
            Self::ReadDirectory { path, .. } => {
                formatter.write_str("cannot read fuzz harness directory ")?;
                escaped_path(formatter, path)
            }
            Self::ReadEntry { .. } => formatter.write_str("cannot read fuzz harness entry"),
        }
    }
}

impl Error for TargetError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Inspect { source, .. }
            | Self::ReadDirectory { source, .. }
            | Self::ReadEntry { source } => Some(source),
            Self::Process(source) => Some(source),
            Self::Disagreement { .. }
            | Self::Duplicate
            | Self::EmptyHarnesses
            | Self::EmptyRegistry
            | Self::InvalidEncoding
            | Self::ListFailed { .. }
            | Self::Malformed(_)
            | Self::MalformedPath(_)
            | Self::NonRegular(_) => None,
        }
    }
}

impl From<ProcessError> for TargetError {
    fn from(error: ProcessError) -> Self {
        Self::Process(error)
    }
}

struct Names<'a>(&'a [String]);

impl fmt::Display for Names<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("[")?;
        for (index, target) in self.0.iter().enumerate() {
            if index != 0 {
                formatter.write_str(", ")?;
            }
            write!(formatter, "{target:?}")?;
        }
        formatter.write_str("]")
    }
}
