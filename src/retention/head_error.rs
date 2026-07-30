//! This module owns semantic retention head failures.

use std::{error::Error, fmt};

use super::{LivenessGeneration, RetentionManifestDigest};

/// Failure to construct one semantic retention head.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RetentionHeadError {
    /// Generation one carried an impossible predecessor.
    InitialGenerationHasPredecessor {
        /// Observed predecessor digest.
        observed: RetentionManifestDigest,
    },
    /// A successor generation omitted its required predecessor.
    MissingPredecessor {
        /// Successor generation lacking a predecessor.
        generation: LivenessGeneration,
    },
}

impl fmt::Display for RetentionHeadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InitialGenerationHasPredecessor { observed } => write!(
                formatter,
                "initial retention head has predecessor {:?}",
                observed.as_bytes()
            ),
            Self::MissingPredecessor { generation } => write!(
                formatter,
                "retention head generation {} requires a predecessor",
                generation.get()
            ),
        }
    }
}

impl Error for RetentionHeadError {}
