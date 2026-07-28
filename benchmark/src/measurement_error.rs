//! Typed failures while collecting benchmark measurements.

use std::error::Error;
use std::fmt;
use std::io;

use crate::{Scenario, ScenarioError, ScenarioObservation};

/// Failure to collect exact, bounded benchmark evidence.
pub enum MeasurementError {
    /// Sample count fell outside the admitted bound.
    InvalidSampleCount {
        /// Smallest admitted count.
        minimum: usize,
        /// Largest admitted count.
        maximum: usize,
        /// Supplied count.
        observed: usize,
    },
    /// Warmup count exceeded its admitted bound.
    InvalidWarmupCount {
        /// Largest admitted count.
        maximum: usize,
        /// Supplied count.
        observed: usize,
    },
    /// No scenario was supplied.
    EmptyScenarioSelection,
    /// A scenario coordinate appeared more than once.
    DuplicateScenario {
        /// Repeated coordinate.
        scenario: Scenario,
    },
    /// A bounded measurement collection could not be reserved.
    Allocation {
        /// Collection being reserved.
        target: &'static str,
        /// Original reservation failure.
        source: std::collections::TryReserveError,
    },
    /// Scenario preparation or execution failed.
    Scenario {
        /// Scenario that failed.
        scenario: Scenario,
        /// Original scenario failure.
        source: Box<ScenarioError>,
    },
    /// The process CPU clock failed.
    CpuClock {
        /// Clock operation that failed.
        action: &'static str,
        /// Original operating-system failure.
        source: io::Error,
    },
    /// The measurement callback did not yield a result.
    MissingSampleResult,
    /// Semantic work changed between timed samples.
    NondeterministicObservation {
        /// Scenario whose work changed.
        scenario: Scenario,
        /// First observed work.
        expected: Box<ScenarioObservation>,
        /// Later observed work.
        observed: Box<ScenarioObservation>,
    },
    /// Checked metric arithmetic refused overflow or zero division.
    MetricArithmetic {
        /// Metric being calculated.
        metric: &'static str,
        /// Existing value or numerator.
        current: u128,
        /// Incoming value or denominator.
        incoming: u128,
    },
}

impl fmt::Debug for MeasurementError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, formatter)
    }
}

impl fmt::Display for MeasurementError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSampleCount {
                minimum,
                maximum,
                observed,
            } => write!(
                formatter,
                "benchmark sample count {observed} is outside {minimum}..={maximum}"
            ),
            Self::InvalidWarmupCount { maximum, observed } => write!(
                formatter,
                "benchmark warmup count {observed} exceeds {maximum}"
            ),
            Self::EmptyScenarioSelection => formatter.write_str("benchmark scenario list is empty"),
            Self::DuplicateScenario { scenario } => {
                write!(
                    formatter,
                    "benchmark scenario `{}` is duplicated",
                    scenario.name()
                )
            }
            Self::Allocation { target, .. } => {
                write!(
                    formatter,
                    "could not reserve benchmark measurement {target}"
                )
            }
            Self::Scenario { scenario, .. } => {
                write!(formatter, "benchmark scenario `{}` failed", scenario.name())
            }
            Self::CpuClock { action, .. } => {
                write!(formatter, "could not {action} benchmark process CPU clock")
            }
            Self::MissingSampleResult => {
                formatter.write_str("benchmark measurement callback returned no result")
            }
            Self::NondeterministicObservation { scenario, .. } => write!(
                formatter,
                "benchmark scenario `{}` changed semantic work between samples",
                scenario.name()
            ),
            Self::MetricArithmetic {
                metric,
                current,
                incoming,
            } => write!(
                formatter,
                "benchmark metric `{metric}` cannot combine {current} and {incoming}"
            ),
        }
    }
}

impl Error for MeasurementError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Allocation { source, .. } => Some(source),
            Self::Scenario { source, .. } => Some(source),
            Self::CpuClock { source, .. } => Some(source),
            Self::InvalidSampleCount { .. }
            | Self::InvalidWarmupCount { .. }
            | Self::EmptyScenarioSelection
            | Self::DuplicateScenario { .. }
            | Self::MissingSampleResult
            | Self::NondeterministicObservation { .. }
            | Self::MetricArithmetic { .. } => None,
        }
    }
}
