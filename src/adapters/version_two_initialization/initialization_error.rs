//! This module owns typed direct-initialization failures.

use std::error::Error;
use std::fmt;
use std::io;

use super::{VersionTwoInitializationPhase, VersionTwoInitializationRefusal};
use crate::adapters::{FilesystemPlatformAdmissionError, WriterLockAcquireError};

/// Failure of direct version-two initialization.
///
/// A failure leaves the root in a state that the protocol classifies: either
/// unchanged, or a recoverable prefix that a later call resumes. No variant
/// implies cleanup was attempted.
#[derive(Debug)]
#[non_exhaustive]
pub enum VersionTwoInitializationError {
    /// The root could not be opened or does not satisfy the platform profile.
    Platform {
        /// Preserved platform-admission failure.
        source: io::Error,
    },
    /// Exclusive writer authority could not be acquired.
    WriterLock {
        /// Preserved writer-lock failure, including `Busy` for a concurrent attempt.
        source: WriterLockAcquireError,
    },
    /// The root is not a recoverable initialization prefix.
    Refused {
        /// The exact refusal and its evidence.
        refusal: VersionTwoInitializationRefusal,
    },
    /// One phase failed with an I/O error or a concurrent change it detected.
    Io {
        /// The exact failed phase.
        phase: VersionTwoInitializationPhase,
        /// Preserved storage failure.
        source: io::Error,
    },
    /// The committed store failed version-two admission.
    Admission {
        /// Preserved admission failure.
        source: FilesystemPlatformAdmissionError,
    },
}

impl VersionTwoInitializationError {
    pub(super) const fn refused(refusal: VersionTwoInitializationRefusal) -> Self {
        Self::Refused { refusal }
    }

    pub(super) const fn io(phase: VersionTwoInitializationPhase, source: io::Error) -> Self {
        Self::Io { phase, source }
    }
}

impl fmt::Display for VersionTwoInitializationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Platform { .. } => {
                formatter.write_str("initialization platform admission failed")
            }
            Self::WriterLock { .. } => {
                formatter.write_str("initialization writer-lock acquisition failed")
            }
            Self::Refused { refusal } => write!(formatter, "initialization refused: {refusal}"),
            Self::Io { phase, .. } => write!(formatter, "initialization failed during {phase}"),
            Self::Admission { .. } => {
                formatter.write_str("initialized store failed version-two admission")
            }
        }
    }
}

impl Error for VersionTwoInitializationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Platform { source } | Self::Io { source, .. } => Some(source),
            Self::WriterLock { source } => Some(source),
            Self::Refused { refusal } => Some(refusal),
            Self::Admission { source } => Some(source),
        }
    }
}
