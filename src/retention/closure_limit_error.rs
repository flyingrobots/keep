//! This module owns typed retention closure-limit failures.

use std::error::Error;
use std::fmt;

use super::RetentionClosureLimit;

/// Failure to admit one bounded closure resource.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RetentionClosureLimitError {
    /// A required positive limit was zero.
    Zero {
        /// Resource whose limit was zero.
        limit: RetentionClosureLimit,
    },
    /// A limit exceeded its fixed implementation ceiling.
    AboveMaximum {
        /// Resource whose limit was excessive.
        limit: RetentionClosureLimit,
        /// Fixed implementation ceiling.
        maximum: u64,
        /// Caller-observed limit.
        observed: u64,
    },
}

impl fmt::Display for RetentionClosureLimitError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Zero { limit } => write!(formatter, "retention {limit} limit must be positive"),
            Self::AboveMaximum {
                limit,
                maximum,
                observed,
            } => write!(
                formatter,
                "retention {limit} limit {observed} exceeds maximum {maximum}"
            ),
        }
    }
}

impl Error for RetentionClosureLimitError {}
