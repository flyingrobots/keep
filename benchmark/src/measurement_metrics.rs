//! Read-only access to aggregated scenario measurements.

use crate::measurement::ScenarioMetrics;
use crate::{Scenario, ScenarioObservation};

impl ScenarioMetrics {
    /// Returns the scenario coordinate.
    #[must_use]
    pub const fn scenario(&self) -> Scenario {
        self.observation.scenario()
    }

    /// Returns exact semantic work performed by every sample.
    #[must_use]
    pub const fn observation(&self) -> ScenarioObservation {
        self.observation
    }

    /// Returns the number of timed samples.
    #[must_use]
    pub const fn sample_count(&self) -> usize {
        self.sample_count
    }

    /// Returns total wall duration across timed samples.
    #[must_use]
    pub const fn total_wall_time_ns(&self) -> u128 {
        self.total_wall_time_ns
    }

    /// Returns total process CPU duration across timed samples.
    #[must_use]
    pub const fn total_cpu_time_ns(&self) -> u128 {
        self.total_cpu_time_ns
    }

    /// Returns nearest-rank p50 process CPU latency.
    #[must_use]
    pub const fn p50_cpu_time_ns(&self) -> u128 {
        self.p50_cpu_time_ns
    }

    /// Returns nearest-rank p95 process CPU latency.
    #[must_use]
    pub const fn p95_cpu_time_ns(&self) -> u128 {
        self.p95_cpu_time_ns
    }

    /// Returns nearest-rank p99 process CPU latency.
    #[must_use]
    pub const fn p99_cpu_time_ns(&self) -> u128 {
        self.p99_cpu_time_ns
    }

    /// Returns nearest-rank p50 wall latency.
    #[must_use]
    pub const fn p50_wall_time_ns(&self) -> u128 {
        self.p50_wall_time_ns
    }

    /// Returns nearest-rank p95 wall latency.
    #[must_use]
    pub const fn p95_wall_time_ns(&self) -> u128 {
        self.p95_wall_time_ns
    }

    /// Returns nearest-rank p99 wall latency.
    #[must_use]
    pub const fn p99_wall_time_ns(&self) -> u128 {
        self.p99_wall_time_ns
    }

    /// Returns aggregate logical throughput using total wall duration.
    #[must_use]
    pub const fn logical_bytes_per_second(&self) -> u128 {
        self.logical_bytes_per_second
    }

    /// Returns allocation count summed across samples.
    #[must_use]
    pub const fn total_allocation_count(&self) -> u64 {
        self.total_allocation_count
    }

    /// Returns allocated bytes summed across samples.
    #[must_use]
    pub const fn total_allocated_bytes(&self) -> u64 {
        self.total_allocated_bytes
    }

    /// Returns maximum incremental simultaneous live allocation count.
    #[must_use]
    pub const fn peak_live_allocation_count(&self) -> u64 {
        self.peak_live_allocation_count
    }

    /// Returns maximum incremental live heap bytes in any sample.
    #[must_use]
    pub const fn peak_live_heap_bytes(&self) -> u64 {
        self.peak_live_heap_bytes
    }
}
