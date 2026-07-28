//! Typed failures while constructing or writing a baseline report.

use std::error::Error;
use std::fmt;
use std::io;

use crate::{ChunkingProfile, CorpusError, MeasurementError, ProfileError};

/// Failure to collect or serialize benchmark baseline evidence.
pub enum ReportError {
    /// The corpus could not be generated.
    Corpus(Box<CorpusError>),
    /// Scenario measurement failed.
    Measurement(Box<MeasurementError>),
    /// One profile comparison failed.
    Profile {
        /// Profile that failed.
        profile: ChunkingProfile,
        /// Original profile failure.
        source: Box<ProfileError>,
    },
    /// Repeated profile samples produced different exact partitions.
    NondeterministicProfile {
        /// Profile whose output changed.
        profile: ChunkingProfile,
    },
    /// An environment value was empty or unsafe for TSV.
    InvalidEnvironmentField {
        /// Invalid field coordinate.
        field: &'static str,
    },
    /// An optimized baseline was requested from a debug build.
    DebugBuild,
    /// A required host-environment coordinate could not be read.
    Environment {
        /// Environment operation that failed.
        action: &'static str,
        /// Original operating-system failure.
        source: io::Error,
    },
    /// Report bytes could not be written.
    Write(io::Error),
}

impl fmt::Debug for ReportError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, formatter)
    }
}

impl fmt::Display for ReportError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Corpus(_) => formatter.write_str("could not generate benchmark report corpus"),
            Self::Measurement(_) => formatter.write_str("could not measure benchmark report"),
            Self::Profile { profile, .. } => {
                write!(formatter, "could not compare profile `{}`", profile.name())
            }
            Self::NondeterministicProfile { profile } => write!(
                formatter,
                "profile `{}` changed exact partition between samples",
                profile.name()
            ),
            Self::InvalidEnvironmentField { field } => {
                write!(
                    formatter,
                    "benchmark environment field `{field}` is invalid"
                )
            }
            Self::DebugBuild => formatter.write_str(
                "optimized baseline requires a release build with debug assertions disabled",
            ),
            Self::Environment { action, .. } => {
                write!(formatter, "could not {action} benchmark environment")
            }
            Self::Write(_) => formatter.write_str("could not write benchmark report"),
        }
    }
}

impl Error for ReportError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Corpus(source) => Some(source),
            Self::Measurement(source) => Some(source),
            Self::Profile { source, .. } => Some(source),
            Self::Environment { source, .. } | Self::Write(source) => Some(source),
            Self::NondeterministicProfile { .. }
            | Self::InvalidEnvironmentField { .. }
            | Self::DebugBuild => None,
        }
    }
}

impl From<CorpusError> for ReportError {
    fn from(source: CorpusError) -> Self {
        Self::Corpus(Box::new(source))
    }
}

impl From<MeasurementError> for ReportError {
    fn from(source: MeasurementError) -> Self {
        Self::Measurement(Box::new(source))
    }
}

impl From<io::Error> for ReportError {
    fn from(source: io::Error) -> Self {
        Self::Write(source)
    }
}
