//! Timed profile comparison sequencing and exact partition admission.

use keep::ChunkId;

use crate::measurement::MeasurementSettings;
use crate::measurement_sample::{OperationMetrics, measure_operation};
use crate::report_profile_aggregate::aggregate;
use crate::{
    BenchmarkCorpus, ChunkPartition, ChunkingProfile, MeasurementError, ProfileError, ReportError,
};

pub(super) struct ProfileMetrics {
    pub(super) profile: ChunkingProfile,
    pub(super) sample_count: usize,
    pub(super) total_wall_time_ns: u128,
    pub(super) total_cpu_time_ns: u128,
    pub(super) p50_wall_time_ns: u128,
    pub(super) p95_wall_time_ns: u128,
    pub(super) p99_wall_time_ns: u128,
    pub(super) logical_bytes_per_second: u128,
    pub(super) total_allocation_count: u64,
    pub(super) total_allocated_bytes: u64,
    pub(super) peak_live_heap_bytes: u64,
    pub(super) base_unique_chunks: usize,
    pub(super) base_materialized_bytes: u64,
    pub(super) insertion_reused_chunks: usize,
    pub(super) deletion_reused_chunks: usize,
    pub(super) neighbor_reused_chunks: usize,
}

pub(super) fn measure_profiles(
    corpus: &BenchmarkCorpus,
    settings: MeasurementSettings,
) -> Result<Box<[ProfileMetrics]>, ReportError> {
    let mut profiles = Vec::new();
    profiles
        .try_reserve_exact(ChunkingProfile::ALL.len())
        .map_err(|source| MeasurementError::Allocation {
            target: "profile-results",
            source,
        })?;
    for profile in ChunkingProfile::ALL {
        profiles.push(measure_profile(corpus, settings, profile)?);
    }
    Ok(profiles.into_boxed_slice())
}

fn measure_profile(
    corpus: &BenchmarkCorpus,
    settings: MeasurementSettings,
    profile: ChunkingProfile,
) -> Result<ProfileMetrics, ReportError> {
    for _index in 0..settings.warmup_count {
        let _partition = profile
            .partition(corpus.large_text())
            .map_err(|source| profile_error(profile, source))?;
    }
    let mut samples = Vec::new();
    samples
        .try_reserve_exact(settings.sample_count)
        .map_err(|source| MeasurementError::Allocation {
            target: "profile-samples",
            source,
        })?;
    let mut expected = None;
    for _index in 0..settings.sample_count {
        let sample = measure_operation(|| profile.partition(corpus.large_text()))?
            .map_err(|source| profile_error(profile, source))?;
        admit_partition(profile, &mut expected, &sample.value)?;
        samples.push(sample);
    }
    aggregate(profile, corpus, &samples)
}

fn admit_partition(
    profile: ChunkingProfile,
    expected: &mut Option<Vec<ChunkId>>,
    observed: &ChunkPartition,
) -> Result<(), ReportError> {
    if let Some(expected) = expected {
        if expected.as_slice() != observed.identities() {
            return Err(ReportError::NondeterministicProfile { profile });
        }
        return Ok(());
    }
    let mut identities = Vec::new();
    identities
        .try_reserve_exact(observed.identities().len())
        .map_err(|source| MeasurementError::Allocation {
            target: "expected-profile-identities",
            source,
        })?;
    identities.extend_from_slice(observed.identities());
    *expected = Some(identities);
    Ok(())
}

pub(super) fn profile_error(profile: ChunkingProfile, source: ProfileError) -> ReportError {
    ReportError::Profile {
        profile,
        source: Box::new(source),
    }
}

pub(super) type ProfileSample = OperationMetrics<ChunkPartition>;
