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
    /// The observed stages are unrecoverable ambiguity.
    Plan {
        /// The exact planning refusal.
        source: RetentionRecoveryRefusal,
    },
    /// A recovery step refused; earlier steps' effects remain.
    Execute {
        /// The refused step, the completed prefix, and the storage error.
        source: RetentionRecoveryError,
    },
}

impl fmt::Display for FilesystemRetentionRecoveryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Observe { .. } => "retention recovery could not observe the store",
            Self::Plan { .. } => "retention recovery refused the observed stages",
            Self::Execute { .. } => "a retention recovery step refused",
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
