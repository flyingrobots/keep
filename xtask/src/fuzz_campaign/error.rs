//! This module owns typed fuzz campaign command failures.

use std::error::Error;
use std::fmt;
use std::io;

use crate::diagnostic::escaped_controls;

use super::policy::PolicyError;

pub(crate) enum FuzzCampaignError {
    InvalidArgumentEncoding,
    Output(io::Error),
    Policy(PolicyError),
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
            Self::InvalidArgumentEncoding => {
                formatter.write_str("fuzz campaign argument is not valid Unicode")
            }
            Self::Output(_) => formatter.write_str("cannot write fuzz campaign output"),
            Self::Policy(error) => write!(formatter, "fuzz campaign refused: {error}"),
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
                "usage: cargo xtask fuzz <describe|github-env> \
                 [--profile <smoke|scheduled>]",
            ),
        }
    }
}

impl Error for FuzzCampaignError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Output(source) => Some(source),
            Self::Policy(source) => Some(source),
            Self::InvalidArgumentEncoding
            | Self::UnexpectedArgument(_)
            | Self::UnknownOperation(_)
            | Self::Usage => None,
        }
    }
}

impl From<PolicyError> for FuzzCampaignError {
    fn from(error: PolicyError) -> Self {
        Self::Policy(error)
    }
}
