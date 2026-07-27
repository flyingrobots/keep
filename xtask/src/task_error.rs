//! This module owns command-level error aggregation and stable diagnostics.

use std::error::Error;
use std::fmt;

use crate::fuzz_seed_corpus::FuzzSeedError;
use crate::golden_file_worldline::GoldenError;
use crate::source_structure::SourceStructureError;

pub(super) enum TaskError {
    FuzzSeed(FuzzSeedError),
    Golden(GoldenError),
    InvalidCommandEncoding,
    InvalidExtraArgumentEncoding,
    RepositoryRoot,
    SourceStructure(SourceStructureError),
    UnexpectedArgument(String),
    UnknownCommand(String),
    Usage,
}

impl fmt::Debug for TaskError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, formatter)
    }
}

impl fmt::Display for TaskError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FuzzSeed(error) => write!(formatter, "{error}"),
            Self::Golden(error) => write!(formatter, "{error}"),
            Self::InvalidCommandEncoding => {
                formatter.write_str("xtask command is not valid Unicode")
            }
            Self::InvalidExtraArgumentEncoding => {
                formatter.write_str("unexpected xtask argument is not valid Unicode")
            }
            Self::RepositoryRoot => formatter.write_str("xtask manifest has no repository parent"),
            Self::SourceStructure(error) => write!(formatter, "{error}"),
            Self::UnexpectedArgument(argument) => {
                write!(formatter, "unexpected xtask argument `{argument}`")
            }
            Self::UnknownCommand(command) => {
                write!(formatter, "unknown xtask command `{command}`")
            }
            Self::Usage => formatter.write_str(
                "usage: cargo xtask \
                 <golden-file-worldline-check|prepare-fuzz-corpus|\
                 source-structure-check|verify>",
            ),
        }
    }
}

impl Error for TaskError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::FuzzSeed(error) => Some(error),
            Self::Golden(error) => Some(error),
            Self::SourceStructure(error) => Some(error),
            Self::InvalidCommandEncoding
            | Self::InvalidExtraArgumentEncoding
            | Self::RepositoryRoot
            | Self::UnexpectedArgument(_)
            | Self::UnknownCommand(_)
            | Self::Usage => None,
        }
    }
}

impl From<FuzzSeedError> for TaskError {
    fn from(error: FuzzSeedError) -> Self {
        Self::FuzzSeed(error)
    }
}

impl From<GoldenError> for TaskError {
    fn from(error: GoldenError) -> Self {
        Self::Golden(error)
    }
}

impl From<SourceStructureError> for TaskError {
    fn from(error: SourceStructureError) -> Self {
        Self::SourceStructure(error)
    }
}
