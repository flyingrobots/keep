//! This module owns command-level error aggregation and stable diagnostics.

use std::error::Error;
use std::fmt;

use crate::golden_file_worldline::GoldenError;
use crate::source_structure::SourceStructureError;

pub(super) enum TaskError {
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
            Self::Golden(error) => write!(formatter, "{error}"),
            Self::InvalidCommandEncoding => formatter.write_str("xtask command is not valid UTF-8"),
            Self::InvalidExtraArgumentEncoding => {
                formatter.write_str("unexpected xtask argument is not valid UTF-8")
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
                 <golden-file-worldline-check|source-structure-check|verify>",
            ),
        }
    }
}

impl Error for TaskError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
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
