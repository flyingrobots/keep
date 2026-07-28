//! This module owns exact retained fuzz corpus admission failures.

use std::error::Error;
use std::fmt;
use std::io;
use std::path::PathBuf;

use crate::diagnostic::escaped_path;
use crate::fuzz_campaign::target::TargetError;

pub(crate) enum CorpusError {
    ByteCountBound {
        maximum: u64,
    },
    ByteCountOverflow,
    FileCountBound {
        maximum: u64,
    },
    FileCountOverflow,
    InputBound {
        path: PathBuf,
        maximum: u64,
        observed: u64,
    },
    Inspect {
        path: PathBuf,
        source: io::Error,
    },
    NonRegular(PathBuf),
    ReadDirectory {
        path: PathBuf,
        source: io::Error,
    },
    ReadEntry {
        path: PathBuf,
        source: io::Error,
    },
    RootNotDirectory(PathBuf),
    Target(TargetError),
    UnexpectedTarget(PathBuf),
}

impl fmt::Debug for CorpusError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, formatter)
    }
}

impl fmt::Display for CorpusError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ByteCountBound { maximum } => {
                write!(
                    formatter,
                    "corpus byte count exceeds its {maximum}-byte bound"
                )
            }
            Self::ByteCountOverflow => formatter.write_str("corpus byte count overflowed"),
            Self::FileCountBound { maximum } => {
                write!(
                    formatter,
                    "corpus file count exceeds its {maximum}-file bound"
                )
            }
            Self::FileCountOverflow => formatter.write_str("corpus file count overflowed"),
            Self::InputBound {
                path,
                maximum,
                observed,
            } => {
                formatter.write_str("corpus entry ")?;
                escaped_path(formatter, path)?;
                write!(
                    formatter,
                    " is {observed} bytes and exceeds the {maximum}-byte input bound"
                )
            }
            Self::Inspect { path, .. } => {
                formatter.write_str("cannot inspect corpus path ")?;
                escaped_path(formatter, path)
            }
            Self::NonRegular(path) => {
                formatter.write_str("corpus entry is not a regular file: ")?;
                escaped_path(formatter, path)
            }
            Self::ReadDirectory { path, .. } => {
                formatter.write_str("cannot read corpus directory ")?;
                escaped_path(formatter, path)
            }
            Self::ReadEntry { path, .. } => {
                formatter.write_str("cannot read an entry in corpus directory ")?;
                escaped_path(formatter, path)
            }
            Self::RootNotDirectory(path) => {
                formatter.write_str("corpus root is not a directory: ")?;
                escaped_path(formatter, path)
            }
            Self::Target(error) => write!(formatter, "{error}"),
            Self::UnexpectedTarget(path) => {
                formatter.write_str("unexpected corpus target: ")?;
                escaped_path(formatter, path)
            }
        }
    }
}

impl Error for CorpusError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Inspect { source, .. }
            | Self::ReadDirectory { source, .. }
            | Self::ReadEntry { source, .. } => Some(source),
            Self::Target(source) => Some(source),
            Self::ByteCountBound { .. }
            | Self::ByteCountOverflow
            | Self::FileCountBound { .. }
            | Self::FileCountOverflow
            | Self::InputBound { .. }
            | Self::NonRegular(_)
            | Self::RootNotDirectory(_)
            | Self::UnexpectedTarget(_) => None,
        }
    }
}

impl From<TargetError> for CorpusError {
    fn from(error: TargetError) -> Self {
        Self::Target(error)
    }
}
