//! This module owns ordered next-head finalization failures.

use std::error::Error;
use std::fmt;
use std::io;

use super::{
    RecoveryNextHeadFinalizationStorageError, RecoveryNextHeadFinalizationTarget,
    RecoveryStageEvidence,
};

/// Exact failed phase of recovery next-head finalization.
#[derive(Debug)]
pub enum RecoveryNextHeadFinalizationError {
    /// Durable current state or the candidate view did not verify.
    Verify {
        /// Exact candidate target that was refused.
        target: RecoveryNextHeadFinalizationTarget,
        /// Exact underlying verification failure.
        source: Box<RecoveryNextHeadFinalizationStorageError>,
    },
    /// The exact candidate could not atomically replace durable `HEAD`.
    Replace {
        /// Exact `head.next` evidence that could not be finalized.
        evidence: RecoveryStageEvidence,
        /// Exact underlying replacement failure.
        source: Box<RecoveryNextHeadFinalizationStorageError>,
    },
    /// The root directory could not be synchronized after replacement or retry.
    SynchronizeRoot {
        /// Candidate whose current state was not durably confirmed.
        target: RecoveryNextHeadFinalizationTarget,
        /// Exact underlying synchronization failure.
        source: io::Error,
    },
}

impl fmt::Display for RecoveryNextHeadFinalizationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Verify { target, source } => write!(
                formatter,
                "failed to verify recovery head generation {}: {source}",
                target.generation().get()
            ),
            Self::Replace { evidence, source } => write!(
                formatter,
                "failed to finalize {} evidence: {source}",
                evidence.stage()
            ),
            Self::SynchronizeRoot { target, source } => write!(
                formatter,
                "failed to synchronize recovery head generation {}: {source}",
                target.generation().get()
            ),
        }
    }
}

impl Error for RecoveryNextHeadFinalizationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Verify { source, .. } | Self::Replace { source, .. } => Some(source.as_ref()),
            Self::SynchronizeRoot { source, .. } => Some(source),
        }
    }
}
