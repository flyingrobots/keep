//! This module owns recovery segment-stage classification failures.

use std::error::Error;
use std::fmt;

use super::{RecoveryStageMetadataError, SegmentHeaderError, SegmentReadError};

/// Why complete supplied `current.seg` bytes could not be classified lawfully.
#[derive(Debug)]
pub enum RecoverySegmentStageError {
    /// The caller-supplied slice length cannot fit the protocol coordinate.
    AddressSpace {
        /// Host byte count that could not be represented.
        observed: usize,
    },
    /// The complete stage exceeds the segment-stage protocol maximum.
    Metadata {
        /// Exact metadata-admission refusal.
        source: RecoveryStageMetadataError,
    },
    /// A complete fixed segment header is corrupt or unsupported.
    Header {
        /// Exact header refusal.
        source: SegmentHeaderError,
    },
    /// A complete-looking record or reusable-prefix invariant was refused.
    Record {
        /// Exact record or prefix refusal.
        source: SegmentReadError,
    },
    /// A complete-looking sealed segment was refused.
    Complete {
        /// Exact immutable-segment refusal.
        source: SegmentReadError,
    },
}

impl fmt::Display for RecoverySegmentStageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AddressSpace { observed } => write!(
                formatter,
                "segment-stage length {observed} does not fit the protocol coordinate"
            ),
            Self::Metadata { source } => {
                write!(formatter, "segment-stage metadata was refused: {source}")
            }
            Self::Header { source } => {
                write!(formatter, "segment-stage header was refused: {source}")
            }
            Self::Record { source } => {
                write!(
                    formatter,
                    "segment-stage record prefix was refused: {source}"
                )
            }
            Self::Complete { source } => {
                write!(formatter, "complete segment stage was refused: {source}")
            }
        }
    }
}

impl Error for RecoverySegmentStageError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Metadata { source } => Some(source),
            Self::Header { source } => Some(source),
            Self::Record { source } | Self::Complete { source } => Some(source),
            Self::AddressSpace { .. } => None,
        }
    }
}
