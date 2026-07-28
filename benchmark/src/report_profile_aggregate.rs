//! Checked timing, allocation, and edit-reuse aggregation per profile.

use crate::measurement_percentile::{Percentile, nearest_rank};
use crate::report_profile::{ProfileMetrics, ProfileSample, profile_error};
use crate::{BenchmarkCorpus, ChunkingProfile, MeasurementError, ReportError};

pub(super) fn aggregate(
    profile: ChunkingProfile,
    corpus: &BenchmarkCorpus,
    samples: &[ProfileSample],
) -> Result<ProfileMetrics, ReportError> {
    let mut wall_times = durations(samples)?;
    let total_wall_time_ns = checked_sum_u128(&wall_times, "profile-total-wall-time")?;
    let total_cpu_time_ns = samples.iter().try_fold(0_u128, |total, sample| {
        checked_add_u128(total, sample.cpu_time_ns, "profile-total-cpu-time")
    })?;
    let allocation = allocations(samples)?;
    let reuse = reuse(profile, corpus)?;
    Ok(ProfileMetrics {
        profile,
        sample_count: samples.len(),
        total_wall_time_ns,
        total_cpu_time_ns,
        p50_wall_time_ns: nearest_rank(&mut wall_times, Percentile::P50)?,
        p95_wall_time_ns: nearest_rank(&mut wall_times, Percentile::P95)?,
        p99_wall_time_ns: nearest_rank(&mut wall_times, Percentile::P99)?,
        logical_bytes_per_second: throughput(
            corpus.large_text().len(),
            samples.len(),
            total_wall_time_ns,
        )?,
        total_allocation_count: allocation.count,
        total_allocated_bytes: allocation.bytes,
        peak_live_heap_bytes: allocation.peak,
        base_unique_chunks: reuse.base_unique_chunks,
        base_materialized_bytes: reuse.base_materialized_bytes,
        insertion_reused_chunks: reuse.insertion_reused_chunks,
        deletion_reused_chunks: reuse.deletion_reused_chunks,
        neighbor_reused_chunks: reuse.neighbor_reused_chunks,
    })
}

fn durations(samples: &[ProfileSample]) -> Result<Vec<u128>, MeasurementError> {
    let mut durations = Vec::new();
    durations
        .try_reserve_exact(samples.len())
        .map_err(|source| MeasurementError::Allocation {
            target: "profile-duration-percentiles",
            source,
        })?;
    durations.extend(samples.iter().map(|sample| sample.wall_time_ns));
    Ok(durations)
}

struct AllocationAggregate {
    count: u64,
    bytes: u64,
    peak: u64,
}

fn allocations(samples: &[ProfileSample]) -> Result<AllocationAggregate, MeasurementError> {
    let mut aggregate = AllocationAggregate {
        count: 0,
        bytes: 0,
        peak: 0,
    };
    for sample in samples {
        aggregate.count = checked_add_u64(
            aggregate.count,
            sample.allocations.count_total,
            "profile-allocation-count",
        )?;
        aggregate.bytes = checked_add_u64(
            aggregate.bytes,
            sample.allocations.bytes_total,
            "profile-allocated-bytes",
        )?;
        aggregate.peak = aggregate.peak.max(sample.allocations.bytes_max);
    }
    Ok(aggregate)
}

struct ReuseMetrics {
    base_unique_chunks: usize,
    base_materialized_bytes: u64,
    insertion_reused_chunks: usize,
    deletion_reused_chunks: usize,
    neighbor_reused_chunks: usize,
}

fn reuse(profile: ChunkingProfile, corpus: &BenchmarkCorpus) -> Result<ReuseMetrics, ReportError> {
    let base = profile
        .partition(corpus.edit_base())
        .map_err(|source| profile_error(profile, source))?;
    let insertion = profile
        .partition(corpus.early_insertion())
        .map_err(|source| profile_error(profile, source))?;
    let deletion = profile
        .partition(corpus.early_deletion())
        .map_err(|source| profile_error(profile, source))?;
    let neighbor = profile
        .partition(corpus.near_neighbor())
        .map_err(|source| profile_error(profile, source))?;
    Ok(ReuseMetrics {
        base_unique_chunks: base
            .unique_chunk_count()
            .map_err(|source| profile_error(profile, source))?,
        base_materialized_bytes: base
            .unique_materialized_bytes()
            .map_err(|source| profile_error(profile, source))?,
        insertion_reused_chunks: base
            .reused_unique_chunk_count(&insertion)
            .map_err(|source| profile_error(profile, source))?,
        deletion_reused_chunks: base
            .reused_unique_chunk_count(&deletion)
            .map_err(|source| profile_error(profile, source))?,
        neighbor_reused_chunks: base
            .reused_unique_chunk_count(&neighbor)
            .map_err(|source| profile_error(profile, source))?,
    })
}

fn throughput(
    logical_bytes: usize,
    sample_count: usize,
    wall_time_ns: u128,
) -> Result<u128, MeasurementError> {
    let bytes =
        u128::try_from(logical_bytes).map_err(|_source| MeasurementError::MetricArithmetic {
            metric: "profile-logical-bytes",
            current: u128::MAX,
            incoming: 1,
        })?;
    let samples =
        u128::try_from(sample_count).map_err(|_source| MeasurementError::MetricArithmetic {
            metric: "profile-sample-count",
            current: u128::MAX,
            incoming: 1,
        })?;
    let numerator = bytes
        .checked_mul(samples)
        .and_then(|total| total.checked_mul(1_000_000_000))
        .ok_or(MeasurementError::MetricArithmetic {
            metric: "profile-throughput-numerator",
            current: bytes,
            incoming: samples,
        })?;
    numerator
        .checked_div(wall_time_ns)
        .ok_or(MeasurementError::MetricArithmetic {
            metric: "profile-throughput",
            current: numerator,
            incoming: wall_time_ns,
        })
}

fn checked_sum_u128(values: &[u128], metric: &'static str) -> Result<u128, MeasurementError> {
    values.iter().try_fold(0_u128, |total, value| {
        checked_add_u128(total, *value, metric)
    })
}

fn checked_add_u128(
    current: u128,
    incoming: u128,
    metric: &'static str,
) -> Result<u128, MeasurementError> {
    current
        .checked_add(incoming)
        .ok_or(MeasurementError::MetricArithmetic {
            metric,
            current,
            incoming,
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
