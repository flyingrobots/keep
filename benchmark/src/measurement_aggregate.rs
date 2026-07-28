//! Checked aggregation of timed sample evidence.

use crate::measurement::ScenarioMetrics;
use crate::measurement_percentile::{Percentile, nearest_rank};
use crate::measurement_sample::SampleMetrics;
use crate::{MeasurementError, ScenarioObservation};

pub(super) fn aggregate(
    observation: Option<ScenarioObservation>,
    samples: &[SampleMetrics],
) -> Result<ScenarioMetrics, MeasurementError> {
    let observation = observation.ok_or(MeasurementError::MissingSampleResult)?;
    let mut wall_times = durations(samples, DurationKind::Wall)?;
    let mut cpu_times = durations(samples, DurationKind::Cpu)?;
    let total_wall_time_ns = checked_sum_u128(&wall_times, "total-wall-time")?;
    let total_cpu_time_ns = checked_sum_u128(&cpu_times, "total-cpu-time")?;
    let sample_count = samples.len();
    let allocation = aggregate_allocations(samples)?;
    Ok(ScenarioMetrics {
        observation,
        sample_count,
        total_wall_time_ns,
        total_cpu_time_ns,
        p50_wall_time_ns: nearest_rank(&mut wall_times, Percentile::P50)?,
        p95_wall_time_ns: nearest_rank(&mut wall_times, Percentile::P95)?,
        p99_wall_time_ns: nearest_rank(&mut wall_times, Percentile::P99)?,
        p50_cpu_time_ns: nearest_rank(&mut cpu_times, Percentile::P50)?,
        p95_cpu_time_ns: nearest_rank(&mut cpu_times, Percentile::P95)?,
        p99_cpu_time_ns: nearest_rank(&mut cpu_times, Percentile::P99)?,
        logical_bytes_per_second: throughput(observation, sample_count, total_wall_time_ns)?,
        total_allocation_count: allocation.total_count,
        total_allocated_bytes: allocation.total_bytes,
        peak_live_allocation_count: allocation.peak_count,
        peak_live_heap_bytes: allocation.peak_bytes,
    })
}

#[derive(Clone, Copy)]
enum DurationKind {
    Wall,
    Cpu,
}

fn durations(samples: &[SampleMetrics], kind: DurationKind) -> Result<Vec<u128>, MeasurementError> {
    let mut durations = Vec::new();
    durations
        .try_reserve_exact(samples.len())
        .map_err(|source| MeasurementError::Allocation {
            target: "duration-percentiles",
            source,
        })?;
    durations.extend(samples.iter().map(|sample| match kind {
        DurationKind::Wall => sample.wall_time_ns,
        DurationKind::Cpu => sample.cpu_time_ns,
    }));
    Ok(durations)
}

struct AllocationAggregate {
    total_count: u64,
    total_bytes: u64,
    peak_count: u64,
    peak_bytes: u64,
}

fn aggregate_allocations(
    samples: &[SampleMetrics],
) -> Result<AllocationAggregate, MeasurementError> {
    let mut aggregate = AllocationAggregate {
        total_count: 0,
        total_bytes: 0,
        peak_count: 0,
        peak_bytes: 0,
    };
    for sample in samples {
        aggregate.total_count = checked_add_u64(
            aggregate.total_count,
            sample.allocations.count_total,
            "total-allocation-count",
        )?;
        aggregate.total_bytes = checked_add_u64(
            aggregate.total_bytes,
            sample.allocations.bytes_total,
            "total-allocated-bytes",
        )?;
        aggregate.peak_count = aggregate.peak_count.max(sample.allocations.count_max);
        aggregate.peak_bytes = aggregate.peak_bytes.max(sample.allocations.bytes_max);
    }
    Ok(aggregate)
}

fn checked_sum_u128(values: &[u128], metric: &'static str) -> Result<u128, MeasurementError> {
    values.iter().try_fold(0_u128, |total, incoming| {
        total
            .checked_add(*incoming)
            .ok_or(MeasurementError::MetricArithmetic {
                metric,
                current: total,
                incoming: *incoming,
            })
    })
}

fn checked_add_u64(
    current: u64,
    incoming: u64,
    metric: &'static str,
) -> Result<u64, MeasurementError> {
    current
        .checked_add(incoming)
        .ok_or_else(|| MeasurementError::MetricArithmetic {
            metric,
            current: u128::from(current),
            incoming: u128::from(incoming),
        })
}

fn throughput(
    observation: ScenarioObservation,
    sample_count: usize,
    total_wall_time_ns: u128,
) -> Result<u128, MeasurementError> {
    let samples =
        u128::try_from(sample_count).map_err(|_source| MeasurementError::MetricArithmetic {
            metric: "throughput-sample-count",
            current: u128::MAX,
            incoming: 1,
        })?;
    let logical_bytes = u128::from(observation.logical_bytes())
        .checked_mul(samples)
        .and_then(|bytes| bytes.checked_mul(1_000_000_000))
        .ok_or_else(|| MeasurementError::MetricArithmetic {
            metric: "logical-throughput-numerator",
            current: u128::from(observation.logical_bytes()),
            incoming: samples,
        })?;
    logical_bytes
        .checked_div(total_wall_time_ns)
        .ok_or(MeasurementError::MetricArithmetic {
            metric: "logical-throughput",
            current: logical_bytes,
            incoming: total_wall_time_ns,
        })
}
