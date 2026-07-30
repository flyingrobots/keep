//! This boundary module owns ordered store-migration execution failures.

use std::error::Error;
use std::fmt;
use std::io;

use super::StoreMigrationPhase;

/// Failure before or during ordered version-2 store migration.
#[derive(Debug)]
pub enum StoreMigrationError {
    /// Current version-1 authority could not be revalidated before mutation.
    CurrentVerification {
        /// Preserved storage refusal.
        source: io::Error,
    },
    /// One exact durability phase failed.
    Storage {
        /// Phase attempted when storage refused.
        phase: StoreMigrationPhase,
        /// Preserved storage refusal.
        source: io::Error,
    },
}

impl fmt::Display for StoreMigrationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CurrentVerification { .. } => {
                formatter.write_str("store-migration authority verification failed")
            }
            Self::Storage { phase, .. } => {
                write!(formatter, "store-migration phase {phase} failed")
            }
        }
    }
}

impl Error for StoreMigrationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::CurrentVerification { source } | Self::Storage { source, .. } => Some(source),
        }
    }
}
