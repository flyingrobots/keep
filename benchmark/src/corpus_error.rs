//! Typed failures while generating the fixed benchmark corpus.

use std::collections::TryReserveError;
use std::error::Error;
use std::fmt;

use keep::BlobHashError;

/// Failure to construct the deterministic benchmark corpus.
#[derive(Debug)]
pub enum CorpusError {
    /// A bounded generated buffer could not be reserved.
    Allocation {
        /// Semantic buffer being allocated.
        target: &'static str,
        /// Original allocation failure.
        source: TryReserveError,
    },
    /// A generated member could not be given its exact logical identity.
    Identity {
        /// Canonical member name.
        member: &'static str,
        /// Original identity failure.
        source: BlobHashError,
    },
    /// Aggregate corpus length overflowed the host coordinate.
    TotalLengthOverflow,
    /// Generated corpus exceeded its fixed aggregate bound.
    TotalByteLimitExceeded {
        /// Fixed admitted byte limit.
        limit: usize,
        /// Generated aggregate byte count.
        observed: usize,
    },
    /// A repository-owned edit coordinate did not exist in its fixed source.
    InvalidGeneratedRange {
        /// Semantic edit whose coordinate was invalid.
        target: &'static str,
    },
}

impl fmt::Display for CorpusError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Allocation { target, .. } => {
                write!(
                    formatter,
                    "could not reserve generated corpus buffer {target}"
                )
            }
            Self::Identity { member, .. } => {
                write!(
                    formatter,
                    "could not identify generated corpus member {member}"
                )
            }
            Self::TotalLengthOverflow => formatter.write_str("generated corpus length overflowed"),
            Self::TotalByteLimitExceeded { limit, observed } => write!(
                formatter,
                "generated corpus uses {observed} bytes, exceeding limit {limit}"
            ),
            Self::InvalidGeneratedRange { target } => {
                write!(
                    formatter,
                    "generated corpus edit {target} used an invalid range"
                )
            }
        }
    }
}

impl Error for CorpusError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Allocation { source, .. } => Some(source),
            Self::Identity { source, .. } => Some(source),
            Self::TotalLengthOverflow
            | Self::TotalByteLimitExceeded { .. }
            | Self::InvalidGeneratedRange { .. } => None,
        }
    }
}
