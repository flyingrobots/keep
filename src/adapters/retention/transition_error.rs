//! This boundary module owns exact retention transition planning failures.

use std::{error::Error, fmt};

use crate::retention::RetentionGenerationExpectation;
use crate::{RetentionNamespaceDigest, RetentionRootDigest, RootGeneration, RootGenerationError};

/// Failure to admit one candidate retention root transition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RetentionTransitionError {
    /// The observed namespace state disagreed with the caller expectation.
    StaleGeneration {
        /// Caller-supplied expected state.
        expected: RetentionGenerationExpectation,
        /// Exact observed current generation, or normal absence.
        observed: Option<RootGeneration>,
    },
    /// The candidate named a different namespace from the current root.
    NamespaceMismatch {
        /// Current namespace digest.
        expected: RetentionNamespaceDigest,
        /// Candidate namespace digest.
        observed: RetentionNamespaceDigest,
    },
    /// The current generation has no representable successor.
    GenerationExhausted {
        /// Preserved checked-generation failure.
        source: RootGenerationError,
    },
    /// The candidate generation was not the exact required successor.
    CandidateGeneration {
        /// Required candidate generation.
        expected: RootGeneration,
        /// Observed candidate generation.
        observed: RootGeneration,
    },
    /// The candidate did not name the current root digest.
    CandidatePredecessor {
        /// Required predecessor digest.
        expected: RetentionRootDigest,
        /// Candidate predecessor coordinate.
        observed: Option<RetentionRootDigest>,
    },
}

impl fmt::Display for RetentionTransitionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::StaleGeneration {
                expected: RetentionGenerationExpectation::Absent,
                observed: Some(observed),
            } => write!(
                formatter,
                "retention generation is stale: expected absence, observed generation {}",
                observed.get()
            ),
            Self::StaleGeneration {
                expected: RetentionGenerationExpectation::Current(expected),
                observed: None,
            } => write!(
                formatter,
                "retention generation is stale: expected generation {}, observed absence",
                expected.get()
            ),
            Self::StaleGeneration {
                expected: RetentionGenerationExpectation::Current(expected),
                observed: Some(observed),
            } => write!(
                formatter,
                "retention generation is stale: expected generation {}, observed generation {}",
                expected.get(),
                observed.get()
            ),
            Self::StaleGeneration {
                expected: RetentionGenerationExpectation::Absent,
                observed: None,
            } => formatter.write_str(
                "retention generation stale-state error carried matching absent coordinates",
            ),
            Self::NamespaceMismatch { .. } => {
                formatter.write_str("retention candidate namespace mismatch")
            }
            Self::GenerationExhausted { source } => {
                write!(
                    formatter,
                    "retention root generation is exhausted: {source}"
                )
            }
            Self::CandidateGeneration { expected, observed } => write!(
                formatter,
                "retention candidate generation must be {}, observed {}",
                expected.get(),
                observed.get()
            ),
            Self::CandidatePredecessor { .. } => {
                formatter.write_str("retention candidate predecessor digest mismatch")
            }
        }
    }
}

impl Error for RetentionTransitionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::GenerationExhausted { source } => Some(source),
            Self::StaleGeneration { .. }
            | Self::NamespaceMismatch { .. }
            | Self::CandidateGeneration { .. }
            | Self::CandidatePredecessor { .. } => None,
        }
    }
}
