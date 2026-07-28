//! One timed, allocation-observed, authenticated workload sample.

use std::time::Instant;

use allocation_counter::{AllocationInfo, measure};
use cpu_time::ProcessTime;

use crate::{MeasurementError, PreparedScenario, ScenarioObservation};

pub(super) struct SampleMetrics {
    pub(super) observation: ScenarioObservation,
    pub(super) wall_time_ns: u128,
    pub(super) cpu_time_ns: u128,
    pub(super) allocations: AllocationInfo,
}

pub(super) fn measure_sample(
    prepared: &mut PreparedScenario<'_>,
) -> Result<SampleMetrics, MeasurementError> {
    let cpu_start = ProcessTime::try_now().map_err(|source| MeasurementError::CpuClock {
        action: "start",
        source,
    })?;
    let wall_start = Instant::now();
    let mut result = None;
    let allocations = measure(|| {
        result = Some(prepared.run());
    });
    let wall_time_ns = wall_start.elapsed().as_nanos();
    let cpu_time_ns = cpu_start
        .try_elapsed()
        .map_err(|source| MeasurementError::CpuClock {
            action: "stop",
            source,
        })?
        .as_nanos();
    let observation = result
        .ok_or(MeasurementError::MissingSampleResult)?
        .map_err(|source| MeasurementError::Scenario {
            scenario: prepared.scenario(),
            source: Box::new(source),
        })?;
    Ok(SampleMetrics {
        observation,
        wall_time_ns,
        cpu_time_ns,
        allocations,
    })
}
