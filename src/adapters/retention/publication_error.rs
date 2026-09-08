//! This boundary module owns retention publication execution failures.

use std::error::Error;
use std::fmt;
use std::io;

use super::{RetentionPublicationPhase, RetentionTransitionDisposition};

/// Failure before or during ordered retention publication.
#[derive(Debug)]
pub enum RetentionPublicationError {
    /// Current authority could not be revalidated before mutation.
    CurrentVerification {
        /// Preserved storage refusal.
        source: io::Error,
    },
    /// Storage requested publication from an already-committed preparation.
    DispositionMismatch {
        /// Disposition proven during storage-independent preparation.
        prepared: RetentionTransitionDisposition,
        /// Disposition observed under current writer authority.
        observed: RetentionTransitionDisposition,
    },
    /// A publish disposition lacked its private canonical artifacts.
    MissingPublicationArtifacts,
    /// One exact durability phase failed.
    Storage {
        /// Phase attempted when storage refused.
        phase: RetentionPublicationPhase,
        /// Preserved storage refusal.
        source: io::Error,
    },
}

impl fmt::Display for RetentionPublicationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CurrentVerification { .. } => {
                formatter.write_str("retention publication authority verification failed")
            }
            Self::DispositionMismatch { .. } => {
                formatter.write_str("retention publication disposition changed inconsistently")
            }
            Self::MissingPublicationArtifacts => {
                formatter.write_str("retention publication artifacts are missing")
            }
            Self::Storage { phase, .. } => {
                write!(formatter, "retention publication phase {phase} failed")
            }
        }
    }
}

impl Error for RetentionPublicationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::CurrentVerification { source } | Self::Storage { source, .. } => Some(source),
            Self::DispositionMismatch { .. } | Self::MissingPublicationArtifacts => None,
        }
    }
}
