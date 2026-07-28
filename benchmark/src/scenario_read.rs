//! Authenticated range and whole-blob workload execution.

use crate::counting_writer::CountingWriter;
use crate::scenario_ingest_operation::to_u64;
use crate::scenario_observation::WorkCounters;
use crate::scenario_range_metrics::PreparedRange;
use crate::{Scenario, ScenarioError, ScenarioObservation, VerificationPosture};
use keep::{AdmittedLayout, BlobId, ReferenceStore};

pub(super) fn run_ranges(
    scenario: Scenario,
    store: &ReferenceStore,
    target: BlobId,
    ranges: &[PreparedRange],
) -> Result<ScenarioObservation, ScenarioError> {
    let mut observation = ScenarioObservation::new(scenario, VerificationPosture::SelectedChunks);
    for range in ranges.iter().copied() {
        let mut output = CountingWriter::default();
        let _receipt = store
            .read_range(target, range.requested(), &mut output)
            .map_err(|source| ScenarioError::RangeRead {
                scenario,
                source: Box::new(source),
            })?;
        let counters = WorkCounters {
            logical_bytes: range.requested().length().get(),
            authenticated_chunk_bytes_read: range.authenticated_bytes(),
            output_bytes_written: output.bytes_written(),
            chunk_instances: range.chunk_instances(),
            operation_count: 1,
            ..WorkCounters::default()
        };
        observation.add(counters)?;
    }
    Ok(observation)
}

pub(super) fn run_verification(
    scenario: Scenario,
    store: &ReferenceStore,
    target: BlobId,
    layout: &AdmittedLayout,
) -> Result<ScenarioObservation, ScenarioError> {
    let mut output = CountingWriter::default();
    let receipt =
        store
            .reconstruct(target, &mut output)
            .map_err(|source| ScenarioError::Reconstruction {
                scenario,
                source: Box::new(source),
            })?;
    let logical_bytes = receipt.bytes_written().get();
    let authenticated_chunk_bytes_read =
        logical_bytes
            .checked_mul(2)
            .ok_or(ScenarioError::MetricOverflow {
                metric: "authenticated-chunk-bytes-read",
                current: logical_bytes,
                incoming: logical_bytes,
            })?;
    let mut observation = ScenarioObservation::new(scenario, VerificationPosture::CompleteBlob);
    observation.add(WorkCounters {
        logical_bytes,
        authenticated_chunk_bytes_read,
        output_bytes_written: output.bytes_written(),
        chunk_instances: to_u64(layout.entries().len(), "chunk-instances")?,
        operation_count: 1,
        ..WorkCounters::default()
    })?;
    Ok(observation)
}
