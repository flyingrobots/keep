//! This module owns typed fuzz campaign command failures.

use std::error::Error;
use std::fmt;
use std::io;

use crate::diagnostic::escaped_controls;

use super::corpus::CorpusError;
use super::execution::ExecutionError;
use super::policy::PolicyError;
use super::target::TargetError;

pub(crate) enum FuzzCampaignError {
    Corpus(CorpusError),
    Execution(ExecutionError),
    InvalidArgumentEncoding,
    Output(io::Error),
    Policy(PolicyError),
    Target(TargetError),
    UnexpectedArgument(String),
    UnknownOperation(String),
    Usage,
}

impl fmt::Debug for FuzzCampaignError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, formatter)
    }
}

impl fmt::Display for FuzzCampaignError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Corpus(error) => write!(formatter, "fuzz corpus refused: {error}"),
            Self::Execution(error) => write!(formatter, "{error}"),
            Self::InvalidArgumentEncoding => {
                formatter.write_str("fuzz campaign argument is not valid Unicode")
            }
            Self::Output(_) => formatter.write_str("cannot write fuzz campaign output"),
            Self::Policy(error) => write!(formatter, "fuzz campaign refused: {error}"),
            Self::Target(error) => write!(formatter, "fuzz campaign refused: {error}"),
            Self::UnexpectedArgument(argument) => {
                formatter.write_str("unexpected fuzz campaign argument `")?;
                escaped_controls(formatter, argument)?;
                formatter.write_str("`")
            }
            Self::UnknownOperation(operation) => {
                formatter.write_str("unknown fuzz campaign operation `")?;
                escaped_controls(formatter, operation)?;
                formatter.write_str("`")
            }
            Self::Usage => formatter.write_str(
                "usage: cargo xtask fuzz \
                 <build|check-corpus|describe|github-env|list|minimize|run> \
                 [--profile <smoke|scheduled>]",
            ),
        }
    }
}

impl Error for FuzzCampaignError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Corpus(source) => Some(source),
            Self::Execution(source) => Some(source),
            Self::Output(source) => Some(source),
            Self::Policy(source) => Some(source),
            Self::Target(source) => Some(source),
            Self::InvalidArgumentEncoding
            | Self::UnexpectedArgument(_)
            | Self::UnknownOperation(_)
            | Self::Usage => None,
        }
    }
}

impl From<CorpusError> for FuzzCampaignError {
    fn from(error: CorpusError) -> Self {
        Self::Corpus(error)
    }
}

impl From<ExecutionError> for FuzzCampaignError {
    fn from(error: ExecutionError) -> Self {
        Self::Execution(error)
    }
}

impl From<PolicyError> for FuzzCampaignError {
    fn from(error: PolicyError) -> Self {
        Self::Policy(error)
    }
}

impl From<TargetError> for FuzzCampaignError {
    fn from(error: TargetError) -> Self {
        Self::Target(error)
    }
}
