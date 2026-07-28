//! Measurement sequencing and checked aggregation.

use crate::measurement::{BaselineMeasurements, MeasurementSettings, ScenarioMetrics};
use crate::measurement_aggregate::aggregate;
use crate::measurement_sample::measure_sample;
use crate::{BenchmarkCorpus, MeasurementError, PreparedScenario, Scenario, ScenarioObservation};

pub(super) fn measure(
    corpus: &BenchmarkCorpus,
    settings: MeasurementSettings,
    scenarios: &[Scenario],
) -> Result<BaselineMeasurements, MeasurementError> {
    validate_scenarios(scenarios)?;
    let mut measured = Vec::new();
    measured
        .try_reserve_exact(scenarios.len())
        .map_err(|source| MeasurementError::Allocation {
            target: "scenario-results",
            source,
        })?;
    for scenario in scenarios.iter().copied() {
        measured.push(measure_scenario(corpus, settings, scenario)?);
    }
    Ok(BaselineMeasurements {
        scenarios: measured.into_boxed_slice(),
    })
}

fn validate_scenarios(scenarios: &[Scenario]) -> Result<(), MeasurementError> {
    if scenarios.is_empty() {
        return Err(MeasurementError::EmptyScenarioSelection);
    }
    for (index, scenario) in scenarios.iter().copied().enumerate() {
        if scenarios
            .iter()
            .take(index)
            .any(|existing| *existing == scenario)
        {
            return Err(MeasurementError::DuplicateScenario { scenario });
        }
    }
    Ok(())
}

fn measure_scenario(
    corpus: &BenchmarkCorpus,
    settings: MeasurementSettings,
    scenario: Scenario,
) -> Result<ScenarioMetrics, MeasurementError> {
    let mut prepared =
        PreparedScenario::new(scenario, corpus).map_err(|source| MeasurementError::Scenario {
            scenario,
            source: Box::new(source),
        })?;
    for _index in 0..settings.warmup_count {
        let _observation = prepared
            .run()
            .map_err(|source| MeasurementError::Scenario {
                scenario,
                source: Box::new(source),
            })?;
    }
    let mut samples = Vec::new();
    samples
        .try_reserve_exact(settings.sample_count)
        .map_err(|source| MeasurementError::Allocation {
            target: "scenario-samples",
            source,
        })?;
    let mut expected = None;
    for _index in 0..settings.sample_count {
        let sample = measure_sample(&mut prepared)?;
        admit_observation(scenario, &mut expected, sample.value)?;
        samples.push(sample);
    }
    aggregate(expected, &samples)
}

fn admit_observation(
    scenario: Scenario,
    expected: &mut Option<ScenarioObservation>,
    observed: ScenarioObservation,
) -> Result<(), MeasurementError> {
    if let Some(expected) = *expected {
        if expected != observed {
            return Err(MeasurementError::NondeterministicObservation {
                scenario,
                expected: Box::new(expected),
                observed: Box::new(observed),
            });
        }
    } else {
        *expected = Some(observed);
    }
    Ok(())
}
