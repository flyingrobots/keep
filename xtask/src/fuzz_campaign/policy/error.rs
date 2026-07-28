//! This module owns typed fuzz policy admission failures.

use std::error::Error;
use std::fmt;
use std::io;
use std::path::PathBuf;

pub(crate) enum PolicyError {
    Bound {
        key: &'static str,
        minimum: u64,
        maximum: u64,
    },
    CampaignOrder,
    CorpusCapacity,
    InvalidInteger(&'static str),
    InvalidToolchain,
    InvalidVersion,
    Key {
        line: usize,
    },
    Line {
        line: usize,
    },
    Missing(Vec<&'static str>),
    Read {
        path: PathBuf,
        source: io::Error,
    },
    Whitespace {
        line: usize,
    },
}

impl fmt::Debug for PolicyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, formatter)
    }
}

impl fmt::Display for PolicyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bound {
                key,
                minimum,
                maximum,
            } => write!(formatter, "{key} is outside [{minimum}, {maximum}]"),
            Self::CampaignOrder => {
                formatter.write_str("scheduled fuzzing must exceed the smoke budget")
            }
            Self::CorpusCapacity => {
                formatter.write_str("corpus bytes cannot be smaller than one input")
            }
            Self::InvalidInteger(key) => {
                write!(formatter, "{key} is not an ASCII decimal integer")
            }
            Self::InvalidToolchain => formatter.write_str("FUZZ_TOOLCHAIN is not a dated nightly"),
            Self::InvalidVersion => {
                formatter.write_str("CARGO_FUZZ_VERSION is not an exact version")
            }
            Self::Key { line } => {
                write!(
                    formatter,
                    "line {line} has an unknown, duplicate, or empty key"
                )
            }
            Self::Line { line } => write!(formatter, "line {line} is not KEY=VALUE"),
            Self::Missing(keys) => write!(formatter, "campaign policy is missing: {keys:?}"),
            Self::Read { path, .. } => write!(formatter, "cannot read {}", path.display()),
            Self::Whitespace { line } => {
                write!(formatter, "line {line} contains whitespace")
            }
        }
    }
}

impl Error for PolicyError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Read { source, .. } => Some(source),
            Self::Bound { .. }
            | Self::CampaignOrder
            | Self::CorpusCapacity
            | Self::InvalidInteger(_)
            | Self::InvalidToolchain
            | Self::InvalidVersion
            | Self::Key { .. }
            | Self::Line { .. }
            | Self::Missing(_)
            | Self::Whitespace { .. } => None,
        }
    }
}
