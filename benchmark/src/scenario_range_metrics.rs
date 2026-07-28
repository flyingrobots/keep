//! Exact authenticated-byte and chunk accounting for range reads.

use keep::{AdmittedLayout, ByteRange};

use crate::scenario_ingest_operation::to_u64;
use crate::{Scenario, ScenarioError};

pub(super) fn authenticated_range_bytes(
    scenario: Scenario,
    layout: &AdmittedLayout,
    requested: ByteRange,
) -> Result<u64, ScenarioError> {
    let plan = layout
        .plan_range(requested)
        .map_err(|source| ScenarioError::RangePlan {
            scenario,
            source: Box::new(source),
        })?;
    let (first, end) = planned_interval(plan)?;
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

pub(super) fn selected_entry_count(
    scenario: Scenario,
    layout: &AdmittedLayout,
    requested: ByteRange,
) -> Result<u64, ScenarioError> {
    let plan = layout
        .plan_range(requested)
        .map_err(|source| ScenarioError::RangePlan {
            scenario,
            source: Box::new(source),
        })?;
    let (_first, end) = planned_interval(plan)?;
    let count = end.checked_sub(plan.first_entry().unwrap_or(end)).ok_or(
        ScenarioError::MetricOverflow {
            metric: "selected-entry-count",
            current: to_u64(end, "selected-entry-end")?,
            incoming: to_u64(plan.first_entry().unwrap_or(end), "selected-entry-first")?,
        },
    )?;
    to_u64(count, "selected-entry-count")
}

fn planned_interval(plan: keep::RangePlan) -> Result<(usize, usize), ScenarioError> {
    let first = plan.first_entry().unwrap_or(0);
    let end = first
        .checked_add(plan.entry_count())
        .ok_or(ScenarioError::MetricOverflow {
            metric: "planned-entry-end",
            current: to_u64(first, "planned-entry-first")?,
            incoming: to_u64(plan.entry_count(), "planned-entry-count")?,
        })?;
    Ok((first, end))
}
