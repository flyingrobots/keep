//! One verified streaming ingestion operation and its exact counters.

use keep::{LayoutEntry, LayoutEntryLimit, ReferenceStore};

use crate::metered_reader::MeteredReader;
use crate::scenario_observation::WorkCounters;
use crate::{Scenario, ScenarioError, ScenarioObservation};

const STORE_CAPACITY: usize = 67_108_864;
pub(super) const DEFAULT_PARTITION: &[usize] = &[8_192];
pub(super) const TINY_PARTITION: &[usize] = &[17, 3, 64];

pub(super) fn ingest(
    store: &mut ReferenceStore,
    scenario: Scenario,
    source: &[u8],
    widths: &'static [usize],
    observation: &mut ScenarioObservation,
) -> Result<(), ScenarioError> {
    let mut reader = MeteredReader::new(source, widths)?;
    let staged = store
        .stage(&mut reader, LayoutEntryLimit::MAXIMUM)
        .map_err(|source| ScenarioError::Ingestion {
            scenario,
            source: Box::new(source),
        })?;
    let unique_chunks = unique_chunks(staged.layout().entries())?;
    let pending_chunks = to_u64(staged.pending_chunk_count(), "pending-chunk-count")?;
    let pending_materialized_bytes = staged.pending_materialized_bytes();
    let reused_unique_chunks =
        unique_chunks
            .checked_sub(pending_chunks)
            .ok_or(ScenarioError::MetricOverflow {
                metric: "reused-unique-chunks",
                current: unique_chunks,
                incoming: pending_chunks,
            })?;
    let counters = WorkCounters {
        logical_bytes: to_u64(source.len(), "logical-bytes")?,
        source_bytes_read: reader.bytes_read(),
        authenticated_chunk_bytes_read: compared_chunk_bytes(
            source.len(),
            pending_materialized_bytes,
        )?,
        materialized_bytes_written: to_u64(
            pending_materialized_bytes,
            "materialized-bytes-written",
        )?,
        chunk_instances: to_u64(staged.layout().entries().len(), "chunk-instances")?,
        reused_unique_chunks,
        operation_count: 1,
        ..WorkCounters::default()
    };
    let _published = staged
        .commit(store)
        .map_err(|source| ScenarioError::Publication {
            scenario,
            source: Box::new(source),
        })?;
    observation.add(counters)
}

fn compared_chunk_bytes(logical: usize, pending: usize) -> Result<u64, ScenarioError> {
    // Staging compares every chunk occurrence except the first occurrence of
    // each newly materialized identity; `pending` is exactly that byte sum.
    let logical = to_u64(logical, "logical-bytes")?;
    let pending = to_u64(pending, "materialized-bytes-written")?;
    logical
        .checked_sub(pending)
        .ok_or(ScenarioError::MetricOverflow {
            metric: "authenticated-chunk-bytes-read",
            current: logical,
            incoming: pending,
        })
}

fn unique_chunks(entries: &[LayoutEntry]) -> Result<u64, ScenarioError> {
    entries
        .iter()
        .enumerate()
        .filter(|(index, entry)| {
            entries
                .iter()
                .take(*index)
                .all(|seen| seen.chunk_id() != entry.chunk_id())
        })
        .try_fold(0_u64, |count, _entry| {
            count.checked_add(1).ok_or(ScenarioError::MetricOverflow {
                metric: "unique-chunk-count",
                current: count,
                incoming: 1,
            })
        })
}

pub(super) const fn new_store() -> ReferenceStore {
    ReferenceStore::new(keep::ReferenceStoreCapacity::new(STORE_CAPACITY))
}

pub(super) fn to_u64(value: usize, metric: &'static str) -> Result<u64, ScenarioError> {
    u64::try_from(value).map_err(|_source| ScenarioError::MetricOverflow {
        metric,
        current: u64::MAX,
        incoming: 1,
    })
}
