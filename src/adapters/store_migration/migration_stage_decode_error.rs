//! This module owns the original decoder failure of a complete migration stage.

use std::error::Error;
use std::fmt;

use super::{
    StoreFormatMarkerDecodeError, StoreMigrationIntentDecodeError, StoreMigrationReceiptDecodeError,
};

/// The exact typed decoder failure of a complete fixed stage.
#[derive(Debug)]
pub enum StoreMigrationStageDecodeError {
    /// An intent stage failed canonical admission.
    Intent {
        /// Original decoder failure.
        source: StoreMigrationIntentDecodeError,
    },
    /// A marker stage failed canonical admission.
    Marker {
        /// Original decoder failure.
        source: StoreFormatMarkerDecodeError,
    },
    /// A receipt stage failed canonical admission or record binding.
    Receipt {
        /// Original decoder failure.
        source: StoreMigrationReceiptDecodeError,
    },
}

impl fmt::Display for StoreMigrationStageDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Intent { source } => write!(formatter, "intent stage: {source}"),
            Self::Marker { source } => write!(formatter, "marker stage: {source}"),
            Self::Receipt { source } => write!(formatter, "receipt stage: {source}"),
        }
    }
}

impl Error for StoreMigrationStageDecodeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Intent { source } => Some(source),
            Self::Marker { source } => Some(source),
            Self::Receipt { source } => Some(source),
        }
    }
}
