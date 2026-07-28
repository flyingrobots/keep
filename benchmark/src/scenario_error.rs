//! Typed benchmark scenario preparation and execution failures.

use std::collections::TryReserveError;
use std::error::Error;
use std::fmt;

use keep::{
    ByteRangeError, IngestionError, PublishError, RangePlanError, RangeReadError,
    ReconstructionError,
};

use crate::Scenario;

/// Failure to prepare or execute one benchmark scenario.
#[derive(Debug)]
pub enum ScenarioError {
    /// Streaming ingestion refused source bytes.
    Ingestion {
        /// Active scenario.
        scenario: Scenario,
        /// Original ingestion refusal.
        source: Box<IngestionError>,
    },
    /// Explicit reference-store publication failed.
    Publication {
        /// Active scenario.
        scenario: Scenario,
        /// Original publication refusal.
        source: Box<PublishError>,
    },
    /// Authenticated whole-blob reconstruction failed.
    Reconstruction {
        /// Active scenario.
        scenario: Scenario,
        /// Original reconstruction refusal.
        source: Box<ReconstructionError>,
    },
    /// Authenticated range reading failed.
    RangeRead {
        /// Active scenario.
        scenario: Scenario,
        /// Original range-read refusal.
        source: Box<RangeReadError>,
    },
    /// A fixed benchmark coordinate could not construct a range.
    ByteRange {
        /// Active scenario.
        scenario: Scenario,
        /// Original coordinate refusal.
        source: Box<ByteRangeError>,
    },
    /// A fixed range could not be planned against its committed layout.
    RangePlan {
        /// Active scenario.
        scenario: Scenario,
        /// Original planning refusal.
        source: Box<RangePlanError>,
    },
    /// A bounded scenario vector could not be reserved.
    Allocation {
        /// Semantic vector being allocated.
        target: &'static str,
        /// Original allocation failure.
        source: TryReserveError,
    },
    /// A scenario metric overflowed its explicit coordinate.
    MetricOverflow {
        /// Semantic metric being accumulated.
        metric: &'static str,
        /// Value before the attempted addition.
        current: u64,
        /// Attempted addition.
        incoming: u64,
    },
    /// A fixed generated corpus coordinate was unavailable.
    CorpusRangeUnavailable {
        /// Semantic range requested.
        target: &'static str,
        /// Available source bytes.
        available: usize,
    },
}

impl fmt::Display for ScenarioError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ingestion { scenario, .. } => {
                write!(formatter, "scenario {} could not ingest", scenario.name())
            }
            Self::Publication { scenario, .. } => {
                write!(formatter, "scenario {} could not publish", scenario.name())
            }
            Self::Reconstruction { scenario, .. } => write!(
                formatter,
                "scenario {} could not reconstruct",
                scenario.name()
            ),
            Self::RangeRead { scenario, .. } => {
                write!(
                    formatter,
                    "scenario {} could not read a range",
                    scenario.name()
                )
            }
            Self::ByteRange { scenario, .. } => write!(
                formatter,
                "scenario {} has an invalid fixed range",
                scenario.name()
            ),
            Self::RangePlan { scenario, .. } => write!(
                formatter,
                "scenario {} could not plan a fixed range",
                scenario.name()
            ),
            Self::Allocation { target, .. } => {
                write!(formatter, "could not reserve benchmark scenario {target}")
            }
            Self::MetricOverflow {
                metric,
                current,
                incoming,
            } => write!(
                formatter,
                "scenario metric {metric} cannot add {incoming} to {current}"
            ),
            Self::CorpusRangeUnavailable { target, available } => write!(
                formatter,
                "scenario corpus range {target} is unavailable in {available} bytes"
            ),
        }
    }
}

impl Error for ScenarioError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Ingestion { source, .. } => Some(source.as_ref()),
            Self::Publication { source, .. } => Some(source.as_ref()),
            Self::Reconstruction { source, .. } => Some(source.as_ref()),
            Self::RangeRead { source, .. } => Some(source.as_ref()),
            Self::ByteRange { source, .. } => Some(source.as_ref()),
            Self::RangePlan { source, .. } => Some(source.as_ref()),
            Self::Allocation { source, .. } => Some(source),
            Self::MetricOverflow { .. } | Self::CorpusRangeUnavailable { .. } => None,
        }
    }
}
