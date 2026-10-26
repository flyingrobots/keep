//! This boundary module owns semantic GC retirement intent failures.

use std::error::Error;
use std::fmt;

/// Failure to admit one semantic GC retirement intent.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GcRetirementIntentError {
    /// An intent must name at least one candidate.
    NoCandidates,
    /// The candidate set exceeded the fixed maximum.
    CandidateCountExceeded {
        /// Fixed maximum count.
        maximum: u32,
        /// Observed count.
        observed: u32,
    },
    /// Two candidates named one segment.
    DuplicateCandidate {
        /// Zero-based index of the repeated candidate.
        index: u32,
    },
    /// Candidates were not in ascending segment-digest order.
    NonCanonicalCandidateOrder {
        /// Zero-based index of the out-of-order candidate.
        index: u32,
    },
}

impl fmt::Display for GcRetirementIntentError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoCandidates => formatter.write_str("GC retirement intent names no candidate"),
            Self::CandidateCountExceeded { maximum, observed } => write!(
                formatter,
                "GC retirement intent names {observed} candidates; maximum {maximum}"
            ),
            Self::DuplicateCandidate { index } => {
                write!(formatter, "GC candidate {index} repeats a segment")
            }
            Self::NonCanonicalCandidateOrder { index } => {
                write!(
                    formatter,
                    "GC candidate {index} breaks segment-digest order"
                )
            }
        }
    }
}

impl Error for GcRetirementIntentError {}
