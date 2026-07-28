//! One timed, allocation-observed, authenticated workload sample.

use std::time::Instant;

use allocation_counter::{AllocationInfo, measure};
use cpu_time::ProcessTime;

use crate::{MeasurementError, PreparedScenario, ScenarioObservation};

pub(super) struct OperationMetrics<T> {
    pub(super) value: T,
    pub(super) wall_time_ns: u128,
    pub(super) cpu_time_ns: u128,
    pub(super) allocations: AllocationInfo,
}

pub(super) type SampleMetrics = OperationMetrics<ScenarioObservation>;

pub(super) fn measure_sample(
    prepared: &mut PreparedScenario<'_>,
) -> Result<SampleMetrics, MeasurementError> {
    measure_operation(|| prepared.run())?.map_err(|source| MeasurementError::Scenario {
        scenario: prepared.scenario(),
        source: Box::new(source),
    })
}

pub(super) fn measure_operation<T, E>(
    operation: impl FnOnce() -> Result<T, E>,
) -> Result<Result<OperationMetrics<T>, E>, MeasurementError> {
    let cpu_start = ProcessTime::try_now().map_err(|source| MeasurementError::CpuClock {
        action: "start",
        source,
    })?;
    let wall_start = Instant::now();
    let mut result = None;
    let allocations = measure(|| {
        result = Some(operation());
    });
    let wall_time_ns = wall_start.elapsed().as_nanos();
    let cpu_time_ns = cpu_start
        .try_elapsed()
        .map_err(|source| MeasurementError::CpuClock {
            action: "stop",
            source,
        })?
        .as_nanos();
    Ok(result
        .ok_or(MeasurementError::MissingSampleResult)?
        .map(|value| OperationMetrics {
            value,
            wall_time_ns,
            cpu_time_ns,
            allocations,
        }))
}
