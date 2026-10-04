//! This module owns candidate-head stage classification failures.

use std::error::Error;
use std::fmt;

use super::{PublicationHeadDecodeError, RecoveryStageMetadataError};

/// Why supplied `head.next` bytes could not be classified lawfully.
#[derive(Debug)]
pub enum RecoveryNextHeadStageError {
    /// The caller-supplied slice length cannot fit the protocol coordinate.
    AddressSpace {
        /// Host byte count that could not be represented.
        observed: usize,
    },
    /// The complete stage exceeds the fixed candidate-head width.
    Metadata {
        /// Exact metadata-admission refusal.
        source: RecoveryStageMetadataError,
    },
    /// Available fixed framing or complete candidate-head bytes were refused.
    Complete {
        /// Exact canonical publication-head refusal.
        source: PublicationHeadDecodeError,
    },
}

impl fmt::Display for RecoveryNextHeadStageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AddressSpace { observed } => write!(
                formatter,
                "next-head length {observed} does not fit the protocol coordinate"
            ),
            Self::Metadata { source } => {
                write!(formatter, "next-head metadata was refused: {source}")
            }
            Self::Complete { source } => {
                write!(formatter, "next-head stage was refused: {source}")
            }
        }
    }
}

impl Error for RecoveryNextHeadStageError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Metadata { source } => Some(source),
            Self::Complete { source } => Some(source),
            Self::AddressSpace { .. } => None,
        }
    }
}
