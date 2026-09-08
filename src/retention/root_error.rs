//! This module owns typed semantic retention root failures.

use std::error::Error;
use std::fmt;

use super::{RetentionAnchor, RetentionRootDigest, RootGeneration};

/// Failure to construct one canonical semantic retention root.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RetentionRootError {
    /// Generation one carried an impossible predecessor.
    InitialGenerationHasPredecessor {
        /// Observed predecessor digest.
        observed: RetentionRootDigest,
    },
    /// A successor generation omitted its required predecessor.
    MissingPredecessor {
        /// Successor generation lacking a predecessor.
        generation: RootGeneration,
    },
    /// The caller supplied too many anchors.
    AnchorCountExceeded {
        /// Fixed maximum anchor count.
        maximum: u32,
        /// Observed anchor count.
        observed: usize,
    },
    /// The caller supplied the same anchor more than once.
    DuplicateAnchor {
        /// Exact duplicated anchor.
        anchor: RetentionAnchor,
    },
}

impl fmt::Display for RetentionRootError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InitialGenerationHasPredecessor { observed } => write!(
                formatter,
                "initial retention root has predecessor {:?}",
                observed.as_bytes()
            ),
            Self::MissingPredecessor { generation } => write!(
                formatter,
                "retention root generation {} requires a predecessor",
                generation.get()
            ),
            Self::AnchorCountExceeded { maximum, observed } => write!(
                formatter,
                "retention root has {observed} anchors; maximum is {maximum}"
            ),
            Self::DuplicateAnchor { anchor } => {
                write!(formatter, "retention root repeats anchor {anchor:?}")
            }
        }
    }
}

impl Error for RetentionRootError {}
