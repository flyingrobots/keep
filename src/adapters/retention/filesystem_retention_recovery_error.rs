//! This module owns the typed error of filesystem retention recovery.

use std::error::Error;
use std::fmt;
use std::io;

use super::{RetentionRecoveryError, RetentionRecoveryRefusal};

/// Why filesystem retention recovery did not reach a receipt.
#[derive(Debug)]
#[non_exhaustive]
pub enum FilesystemRetentionRecoveryError {
    /// Reading the current state, a stage, or a pool entry failed.
    Observe {
        /// The exact filesystem or admission failure.
        source: io::Error,
    },
    /// The observed stages require disposition or contradict the recovery contract.
    Plan {
        /// The exact planning refusal.
        source: RetentionRecoveryRefusal,
    },
    /// A recovery step failed, possibly after its own effects and earlier completed steps.
    Execute {
        /// The failed step, completed prefix, failing-capability progress and original cause.
        source: RetentionRecoveryError,
    },
}

impl fmt::Display for FilesystemRetentionRecoveryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Observe { .. } => "retention recovery could not observe the store",
            Self::Plan { .. } => "retention recovery refused the observed stages",
            Self::Execute { .. } => "a retention recovery step failed",
        })
    }
}

impl Error for FilesystemRetentionRecoveryError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Observe { source } => Some(source),
            Self::Plan { source } => Some(source),
            Self::Execute { source } => Some(source),
        }
    }
}
