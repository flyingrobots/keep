//! This module owns retention manifest length admission failures.

use std::{error::Error, fmt};

/// Failure to admit a canonical retention manifest byte length.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RetentionManifestLengthError {
    /// The length is outside the version-2 manifest bounds.
    OutOfBounds {
        /// Smallest complete manifest length.
        minimum: u64,
        /// Largest permitted manifest length.
        maximum: u64,
        /// Length supplied by the boundary.
        observed: u64,
    },
    /// The length cannot contain a whole number of fixed-width entries.
    NotCongruent {
        /// Length supplied by the boundary.
        observed: u64,
    },
}

impl fmt::Display for RetentionManifestLengthError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OutOfBounds {
                minimum,
                maximum,
                observed,
            } => write!(
                formatter,
                "retention manifest length {observed} is outside {minimum}..={maximum}"
            ),
            Self::NotCongruent { observed } => {
                write!(
                    formatter,
                    "retention manifest length {observed} is not congruent"
                )
            }
        }
    }
}

impl Error for RetentionManifestLengthError {}
