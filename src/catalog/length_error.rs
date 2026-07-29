//! Catalog-length admission failures.

use std::error::Error;
use std::fmt;

/// Failure to admit a canonical catalog byte length.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CatalogLengthError {
    /// The length is outside the version-1 catalog bounds.
    OutOfBounds {
        /// Smallest complete catalog length.
        minimum: u64,
        /// Largest permitted catalog length.
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

impl fmt::Display for CatalogLengthError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OutOfBounds {
                minimum,
                maximum,
                observed,
            } => write!(
                formatter,
                "catalog length {observed} is outside {minimum}..={maximum}"
            ),
            Self::NotCongruent { observed } => {
                write!(formatter, "catalog length {observed} is not congruent")
            }
        }
    }
}

impl Error for CatalogLengthError {}
