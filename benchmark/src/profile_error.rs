//! Typed refusals while calculating benchmark-only partitions.

use std::collections::TryReserveError;
use std::error::Error;
use std::fmt;

use keep::ChunkHashError;

/// Failure to calculate one exact benchmark partition.
#[derive(Debug)]
pub enum ProfileError {
    /// A bounded partition vector could not be reserved.
    Allocation {
        /// Semantic vector being allocated.
        target: &'static str,
        /// Original allocation failure.
        source: TryReserveError,
    },
    /// One generated boundary coordinate overflowed.
    CoordinateOverflow {
        /// Coordinate before the attempted advance.
        current: usize,
        /// Attempted advance.
        incoming: usize,
    },
    /// A derived comparison metric overflowed.
    MetricOverflow {
        /// Semantic metric being accumulated.
        metric: &'static str,
        /// Value before the attempted addition.
        current: u64,
        /// Attempted addition.
        incoming: u64,
    },
    /// A generated boundary did not advance.
    NonIncreasingBoundary {
        /// Prior exclusive end.
        previous: usize,
        /// Observed exclusive end.
        observed: usize,
    },
    /// The final boundary did not consume the complete input.
    FinalBoundaryMismatch {
        /// Exact source length.
        expected: usize,
        /// Last observed exclusive end.
        observed: usize,
    },
    /// A byte could not resolve in a fixed Gear table.
    MissingGearEntry {
        /// Source byte value.
        byte: u8,
    },
    /// A generated chunk identity failed.
    ChunkIdentity {
        /// Inclusive chunk start.
        start: usize,
        /// Exclusive chunk end.
        end: usize,
        /// Original identity failure.
        source: ChunkHashError,
    },
}

impl fmt::Display for ProfileError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Allocation { target, .. } => {
                write!(formatter, "could not reserve benchmark partition {target}")
            }
            Self::CoordinateOverflow { current, incoming } => write!(
                formatter,
                "benchmark coordinate {current} cannot advance by {incoming}"
            ),
            Self::MetricOverflow {
                metric,
                current,
                incoming,
            } => write!(
                formatter,
                "benchmark metric {metric} cannot add {incoming} to {current}"
            ),
            Self::NonIncreasingBoundary { previous, observed } => write!(
                formatter,
                "benchmark boundary {observed} does not follow {previous}"
            ),
            Self::FinalBoundaryMismatch { expected, observed } => write!(
                formatter,
                "benchmark partition ended at {observed}, expected {expected}"
            ),
            Self::MissingGearEntry { byte } => {
                write!(
                    formatter,
                    "benchmark Gear table has no entry for byte {byte}"
                )
            }
            Self::ChunkIdentity { start, end, .. } => {
                write!(
                    formatter,
                    "could not identify benchmark chunk [{start}, {end})"
                )
            }
        }
    }
}

impl Error for ProfileError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Allocation { source, .. } => Some(source),
            Self::ChunkIdentity { source, .. } => Some(source),
            Self::CoordinateOverflow { .. }
            | Self::MetricOverflow { .. }
            | Self::NonIncreasingBoundary { .. }
            | Self::FinalBoundaryMismatch { .. }
            | Self::MissingGearEntry { .. } => None,
        }
    }
}
