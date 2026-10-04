//! This module owns crash-matrix coordinate validation failures.

use std::error::Error;
use std::fmt;

use crate::{DurabilityCrashOccurrence, DurabilityCrashPoint};

/// A failure to construct a valid crash-matrix coordinate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DurabilityCrashCaseError {
    /// A coordinate exceeds a protocol-fixed occurrence range.
    OccurrenceOutOfRange {
        /// The transition with the fixed range.
        point: DurabilityCrashPoint,
        /// The rejected zero-based occurrence.
        observed: DurabilityCrashOccurrence,
        /// The exclusive upper bound for this position.
        exclusive_limit: u32,
    },
    /// A repeated durability transition lacks its occurrence coordinate.
    MissingOccurrence {
        /// The repeated transition missing its coordinate.
        point: DurabilityCrashPoint,
    },
    /// A non-repeated transition received an occurrence coordinate.
    UnexpectedOccurrence {
        /// The transition that cannot accept an occurrence.
        point: DurabilityCrashPoint,
        /// The rejected coordinate.
        observed: DurabilityCrashOccurrence,
    },
}

impl fmt::Display for DurabilityCrashCaseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OccurrenceOutOfRange {
                point,
                observed,
                exclusive_limit,
            } => write!(
                formatter,
                "{} occurrence {} must be less than {exclusive_limit}",
                point.identifier(),
                observed.get()
            ),
            Self::MissingOccurrence { point } => {
                write!(formatter, "{} requires an occurrence", point.identifier())
            }
            Self::UnexpectedOccurrence { point, observed } => write!(
                formatter,
                "{} cannot accept occurrence {}",
                point.identifier(),
                observed.get()
            ),
        }
    }
}

impl Error for DurabilityCrashCaseError {}
