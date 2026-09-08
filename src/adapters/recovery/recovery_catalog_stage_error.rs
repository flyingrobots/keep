//! This module owns recovery catalog-stage classification failures.

use std::error::Error;
use std::fmt;

use super::{CatalogDecodeError, RecoveryStageMetadataError};

/// Why complete supplied `current.cat` bytes could not be classified lawfully.
#[derive(Debug)]
pub enum RecoveryCatalogStageError {
    /// The caller-supplied slice length cannot fit the protocol coordinate.
    AddressSpace {
        /// Host byte count that could not be represented.
        observed: usize,
    },
    /// The complete stage exceeds the catalog-stage protocol maximum.
    Metadata {
        /// Exact metadata-admission refusal.
        source: RecoveryStageMetadataError,
    },
    /// The complete fixed catalog header was refused.
    Header {
        /// Exact catalog-header refusal.
        source: CatalogDecodeError,
    },
    /// Complete-looking catalog bytes were refused.
    Complete {
        /// Exact canonical catalog refusal.
        source: CatalogDecodeError,
    },
}

impl fmt::Display for RecoveryCatalogStageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AddressSpace { observed } => write!(
                formatter,
                "catalog-stage length {observed} does not fit the protocol coordinate"
            ),
            Self::Metadata { source } => {
                write!(formatter, "catalog-stage metadata was refused: {source}")
            }
            Self::Header { source } => {
                write!(formatter, "catalog-stage header was refused: {source}")
            }
            Self::Complete { source } => {
                write!(formatter, "complete catalog stage was refused: {source}")
            }
        }
    }
}

impl Error for RecoveryCatalogStageError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Metadata { source } => Some(source),
            Self::Header { source } | Self::Complete { source } => Some(source),
            Self::AddressSpace { .. } => None,
        }
    }
}
