//! Prepared exact accounting for deterministic range-read requests.

use keep::{AdmittedLayout, ByteRange, RangePlan};

use crate::scenario_ingest_operation::to_u64;
use crate::{Scenario, ScenarioError};

#[derive(Clone, Copy)]
pub(super) struct PreparedRange {
    requested: ByteRange,
    authenticated_bytes: u64,
    chunk_instances: u64,
}

impl PreparedRange {
    pub(super) const fn requested(self) -> ByteRange {
        self.requested
    }

    pub(super) const fn authenticated_bytes(self) -> u64 {
        self.authenticated_bytes
    }

    pub(super) const fn chunk_instances(self) -> u64 {
        self.chunk_instances
    }
}

pub(super) fn prepare(
    scenario: Scenario,
    layout: &AdmittedLayout,
    requests: &[ByteRange],
) -> Result<Box<[PreparedRange]>, ScenarioError> {
    let mut prepared = Vec::new();
    prepared
        .try_reserve_exact(requests.len())
        .map_err(|source| ScenarioError::Allocation {
            target: "prepared-range-accounting",
            source,
        })?;
    for requested in requests.iter().copied() {
        prepared.push(prepare_one(scenario, layout, requested)?);
    }
    Ok(prepared.into_boxed_slice())
}

fn prepare_one(
    scenario: Scenario,
    layout: &AdmittedLayout,
    requested: ByteRange,
) -> Result<PreparedRange, ScenarioError> {
    let plan = layout
        .plan_range(requested)
        .map_err(|source| ScenarioError::RangePlan {
            scenario,
            source: Box::new(source),
        })?;
    Ok(PreparedRange {
        requested,
        authenticated_bytes: authenticated_bytes(layout, plan)?,
        chunk_instances: to_u64(plan.entry_count(), "selected-entry-count")?,
    })
}

fn authenticated_bytes(layout: &AdmittedLayout, plan: RangePlan) -> Result<u64, ScenarioError> {
    let first = plan.first_entry().unwrap_or(0);
    let end = first
        .checked_add(plan.entry_count())
        .ok_or(ScenarioError::MetricOverflow {
            metric: "planned-entry-end",
            current: to_u64(first, "planned-entry-first")?,
            incoming: to_u64(plan.entry_count(), "planned-entry-count")?,
        })?;
    let entries =
        layout
            .entries()
            .get(first..end)
            .ok_or_else(|| ScenarioError::CorpusRangeUnavailable {
                target: "planned-range-entries",
                available: layout.entries().len(),
            })?;
    let once = entries.iter().try_fold(0_u64, |total, entry| {
        let incoming = u64::from(entry.chunk_id().length().get());
        total
            .checked_add(incoming)
            .ok_or(ScenarioError::MetricOverflow {
                metric: "authenticated-range-bytes",
                current: total,
                incoming,
            })
    })?;
    once.checked_mul(2).ok_or(ScenarioError::MetricOverflow {
        metric: "authenticated-range-bytes",
        current: once,
        incoming: once,
    })
}
