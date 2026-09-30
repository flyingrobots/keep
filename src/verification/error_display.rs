//! This module owns verification outcome formatting.

use std::error::Error;
use std::fmt;

use super::{VerificationError, VerificationFailure, VerificationRefusal};

impl fmt::Display for VerificationRefusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing { stage, .. } => {
                write!(
                    formatter,
                    "verification at {stage:?} found required evidence absent"
                )
            }
            Self::Corrupt { stage, .. } => {
                write!(
                    formatter,
                    "verification at {stage:?} found contradicting evidence"
                )
            }
            Self::Ambiguous { stage, .. } => {
                write!(
                    formatter,
                    "verification at {stage:?} found conflicting evidence"
                )
            }
            Self::Unsupported {
                requested,
                supported_minimum,
                supported_maximum,
                ..
            } => write!(
                formatter,
                "verification depth {requested:?} is outside this view's \
                 {supported_minimum:?}..={supported_maximum:?}"
            ),
        }
    }
}

impl Error for VerificationRefusal {}

impl fmt::Display for VerificationFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ChunkHash { index, .. } => {
                write!(formatter, "chunk {index} could not be hashed")
            }
            Self::BlobHash { .. } => formatter.write_str("blob identity could not be calculated"),
            Self::LayoutEncoding { .. } => {
                formatter.write_str("layout could not produce its canonical record")
            }
            Self::ProfileVerifierUnavailable { .. } => {
                formatter.write_str("no verifier implements the registered storage profile")
            }
            Self::ProfileChunking { .. } => {
                formatter.write_str("storage profile replay failed to run")
            }
        }
    }
}

impl Error for VerificationFailure {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ChunkHash { source, .. } => Some(source),
            Self::BlobHash { source } => Some(source),
            Self::LayoutEncoding { source } => Some(source),
            Self::ProfileChunking { source, .. } => Some(source),
            Self::ProfileVerifierUnavailable { .. } => None,
        }
    }
}

impl fmt::Display for VerificationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Refused(refusal) => write!(formatter, "verification refused: {refusal}"),
            Self::Operational(failure) => write!(formatter, "verification failed: {failure}"),
        }
    }
}

impl Error for VerificationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Refused(refusal) => Some(refusal.as_ref()),
            Self::Operational(failure) => Some(failure),
        }
    }
}
