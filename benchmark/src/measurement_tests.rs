//! Laws for repeatable streaming CAS measurement.

use std::error::Error;

use crate::measurement_percentile::{Percentile, nearest_rank};
use crate::{
    BaselineMeasurements, BenchmarkCorpus, MeasurementError, MeasurementSettings, Scenario,
};

#[test]
fn latency_percentiles_use_exact_nearest_rank_coordinates() -> Result<(), MeasurementError> {
    let mut samples = [90_u128, 10, 100, 20, 80, 30, 70, 40, 60, 50];

    assert_eq!(nearest_rank(&mut samples, Percentile::P50)?, 50);
    assert_eq!(nearest_rank(&mut samples, Percentile::P95)?, 100);
    assert_eq!(nearest_rank(&mut samples, Percentile::P99)?, 100);
    Ok(())
}

#[test]
fn measurement_settings_refuse_unbounded_sample_coordinates() {
    assert!(matches!(
        MeasurementSettings::new(0, 0),
        Err(MeasurementError::InvalidSampleCount {
            minimum: 1,
            maximum: 1_000,
            observed: 0
        })
    ));
    assert!(matches!(
        MeasurementSettings::new(1_001, 0),
        Err(MeasurementError::InvalidSampleCount {
            minimum: 1,
            maximum: 1_000,
            observed: 1_001
        })
    ));
    assert!(matches!(
        MeasurementSettings::new(1, 101),
        Err(MeasurementError::InvalidWarmupCount {
            maximum: 100,
            observed: 101
        })
    ));
}

#[test]
fn measured_samples_preserve_semantics_and_record_required_evidence() -> Result<(), Box<dyn Error>>
{
    let corpus = BenchmarkCorpus::generate()?;
    let settings = MeasurementSettings::new(2, 1)?;
    let measurements =
        BaselineMeasurements::measure_selected(&corpus, settings, &[Scenario::ColdIngest])?;
    let scenario = measurements
        .scenarios()
        .first()
        .ok_or("missing measured scenario")?;

    assert_eq!(scenario.scenario(), Scenario::ColdIngest);
    assert_eq!(scenario.sample_count(), 2);
    assert!(scenario.p50_wall_time_ns() > 0);
    assert!(scenario.p95_wall_time_ns() >= scenario.p50_wall_time_ns());
    assert!(scenario.p99_wall_time_ns() >= scenario.p95_wall_time_ns());
    assert!(scenario.total_cpu_time_ns() > 0);
    assert!(scenario.logical_bytes_per_second() > 0);
    assert!(scenario.total_allocation_count() > 0);
    assert!(scenario.total_allocated_bytes() > 0);
    assert!(scenario.peak_live_heap_bytes() > 0);
    assert!(scenario.observation().verification().is_authenticated());
    Ok(())
}
