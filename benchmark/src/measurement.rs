//! Bounded measurement settings and aggregated scenario evidence.

use crate::measurement_run;
use crate::{BenchmarkCorpus, MeasurementError, Scenario, ScenarioObservation};

const MAXIMUM_SAMPLES: usize = 1_000;
const MAXIMUM_WARMUPS: usize = 100;

/// Bounded sample and warmup counts for one benchmark run.
#[derive(Clone, Copy)]
pub struct MeasurementSettings {
    pub(super) sample_count: usize,
    pub(super) warmup_count: usize,
}

/// Complete measurements in stable scenario order.
pub struct BaselineMeasurements {
    pub(super) scenarios: Box<[ScenarioMetrics]>,
}

/// Aggregated timing, allocation, and semantic evidence for one scenario.
pub struct ScenarioMetrics {
    pub(super) observation: ScenarioObservation,
    pub(super) sample_count: usize,
    pub(super) total_wall_time_ns: u128,
    pub(super) total_cpu_time_ns: u128,
    pub(super) p50_wall_time_ns: u128,
    pub(super) p95_wall_time_ns: u128,
    pub(super) p99_wall_time_ns: u128,
    pub(super) p50_cpu_time_ns: u128,
    pub(super) p95_cpu_time_ns: u128,
    pub(super) p99_cpu_time_ns: u128,
    pub(super) logical_bytes_per_second: u128,
    pub(super) total_allocation_count: u64,
    pub(super) total_allocated_bytes: u64,
    pub(super) peak_live_allocation_count: u64,
    pub(super) peak_live_heap_bytes: u64,
}

impl MeasurementSettings {
    /// Creates bounded measurement settings.
    ///
    /// # Errors
    ///
    /// Returns [`MeasurementError`] unless samples are in `1..=1_000` and
    /// warmups are in `0..=100`.
    pub const fn new(sample_count: usize, warmup_count: usize) -> Result<Self, MeasurementError> {
        if sample_count == 0 || sample_count > MAXIMUM_SAMPLES {
            return Err(MeasurementError::InvalidSampleCount {
                minimum: 1,
                maximum: MAXIMUM_SAMPLES,
                observed: sample_count,
            });
        }
        if warmup_count > MAXIMUM_WARMUPS {
            return Err(MeasurementError::InvalidWarmupCount {
                maximum: MAXIMUM_WARMUPS,
                observed: warmup_count,
            });
        }
        Ok(Self {
            sample_count,
            warmup_count,
        })
    }
}

impl BaselineMeasurements {
    /// Measures every required scenario in stable report order.
    ///
    /// # Errors
    ///
    /// Returns [`MeasurementError`] for bounds, setup, workload, clock,
    /// allocation, determinism, or checked-arithmetic failures.
    pub fn measure(
        corpus: &BenchmarkCorpus,
        settings: MeasurementSettings,
    ) -> Result<Self, MeasurementError> {
        Self::measure_selected(corpus, settings, &Scenario::ALL)
    }

    /// Measures an explicit nonempty, duplicate-free scenario selection.
    ///
    /// # Errors
    ///
    /// Returns [`MeasurementError`] for invalid selection, bounds, setup,
    /// workload, clock, allocation, determinism, or arithmetic failures.
    pub fn measure_selected(
        corpus: &BenchmarkCorpus,
        settings: MeasurementSettings,
        scenarios: &[Scenario],
    ) -> Result<Self, MeasurementError> {
        measurement_run::measure(corpus, settings, scenarios)
    }

    /// Returns measurements in the requested stable order.
    #[must_use]
    pub fn scenarios(&self) -> &[ScenarioMetrics] {
        &self.scenarios
    }
}

#[cfg(test)]
#[path = "measurement_tests.rs"]
mod tests;
