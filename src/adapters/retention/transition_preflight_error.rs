//! This boundary module owns retention transition preflight failures.

use std::error::Error;
use std::fmt;

use super::{RetentionClosureVerificationError, RetentionTransitionError};

/// Failure before a retention transition may invoke publication storage.
#[derive(Debug)]
pub enum RetentionTransitionPreflightError {
    /// Generation or exact-successor planning refused the candidate.
    Transition {
        /// Preserved transition-planning refusal.
        source: RetentionTransitionError,
    },
    /// The candidate closure failed against the pinned catalog.
    Closure {
        /// Preserved deterministic closure refusal.
        source: Box<RetentionClosureVerificationError>,
    },
}

impl fmt::Display for RetentionTransitionPreflightError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Transition { .. } => formatter.write_str("retention transition planning failed"),
            Self::Closure { .. } => formatter.write_str("retention closure verification failed"),
        }
    }
}

impl Error for RetentionTransitionPreflightError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Transition { source } => Some(source),
            Self::Closure { source } => Some(source.as_ref()),
        }
    }
}
