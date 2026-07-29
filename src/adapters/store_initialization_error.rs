//! This module owns typed segment-store initialization failures.

use std::error::Error;
use std::fmt;
use std::io;

use super::store_initialization_phase::StoreInitializationPhase;

/// Failure while executing ordered store initialization.
#[derive(Debug)]
pub enum StoreInitializationError {
    /// One initialization phase returned an I/O failure.
    Io {
        /// Exact failed initialization phase.
        phase: StoreInitializationPhase,
        /// Underlying storage failure.
        source: io::Error,
    },
}

impl fmt::Display for StoreInitializationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { phase, .. } => {
                write!(formatter, "store initialization failed during {phase}")
            }
        }
    }
}

impl Error for StoreInitializationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
        }
    }
}
