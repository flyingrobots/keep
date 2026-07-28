//! Exact semantic counters for one integrated benchmark sample.

use crate::{Scenario, ScenarioError, VerificationPosture};

/// Exact semantic work completed by one scenario sample.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScenarioObservation {
    scenario: Scenario,
    verification: VerificationPosture,
    logical_bytes: u64,
    source_bytes_read: u64,
    authenticated_chunk_bytes_read: u64,
    materialized_bytes_written: u64,
    output_bytes_written: u64,
    chunk_instances: u64,
    reused_unique_chunks: u64,
    operation_count: u64,
}

#[derive(Clone, Copy, Default)]
pub(super) struct WorkCounters {
    pub(super) logical_bytes: u64,
    pub(super) source_bytes_read: u64,
    pub(super) authenticated_chunk_bytes_read: u64,
    pub(super) materialized_bytes_written: u64,
    pub(super) output_bytes_written: u64,
    pub(super) chunk_instances: u64,
    pub(super) reused_unique_chunks: u64,
    pub(super) operation_count: u64,
}

impl ScenarioObservation {
    pub(super) const fn new(scenario: Scenario, verification: VerificationPosture) -> Self {
        Self {
            scenario,
            verification,
            logical_bytes: 0,
            source_bytes_read: 0,
            authenticated_chunk_bytes_read: 0,
            materialized_bytes_written: 0,
            output_bytes_written: 0,
            chunk_instances: 0,
            reused_unique_chunks: 0,
            operation_count: 0,
        }
    }

    pub(super) fn add(&mut self, counters: WorkCounters) -> Result<(), ScenarioError> {
        checked_add(
            &mut self.logical_bytes,
            counters.logical_bytes,
            "logical-bytes",
        )?;
        checked_add(
            &mut self.source_bytes_read,
            counters.source_bytes_read,
            "source-bytes-read",
        )?;
        checked_add(
            &mut self.authenticated_chunk_bytes_read,
            counters.authenticated_chunk_bytes_read,
            "authenticated-chunk-bytes-read",
        )?;
        checked_add(
            &mut self.materialized_bytes_written,
            counters.materialized_bytes_written,
            "materialized-bytes-written",
        )?;
        checked_add(
            &mut self.output_bytes_written,
            counters.output_bytes_written,
            "output-bytes-written",
        )?;
        checked_add(
            &mut self.chunk_instances,
            counters.chunk_instances,
            "chunk-instances",
        )?;
        checked_add(
            &mut self.reused_unique_chunks,
            counters.reused_unique_chunks,
            "reused-unique-chunks",
        )?;
        checked_add(
            &mut self.operation_count,
            counters.operation_count,
            "operation-count",
        )
    }

    /// Returns the stable scenario coordinate.
    #[must_use]
    pub const fn scenario(self) -> Scenario {
        self.scenario
    }

    /// Returns the authentication law exercised.
    #[must_use]
    pub const fn verification(self) -> VerificationPosture {
        self.verification
    }

    /// Returns logical bytes processed across all operations.
    #[must_use]
    pub const fn logical_bytes(self) -> u64 {
        self.logical_bytes
    }

    /// Returns exact bytes delivered through caller-owned `Read` boundaries.
    #[must_use]
    pub const fn source_bytes_read(self) -> u64 {
        self.source_bytes_read
    }

    /// Returns complete stored chunk bytes read and authenticated.
    #[must_use]
    pub const fn authenticated_chunk_bytes_read(self) -> u64 {
        self.authenticated_chunk_bytes_read
    }

    /// Returns new exact chunk bytes materialized by ingestion.
    #[must_use]
    pub const fn materialized_bytes_written(self) -> u64 {
        self.materialized_bytes_written
    }

    /// Returns exact bytes passed through caller-owned `Write` boundaries.
    #[must_use]
    pub const fn output_bytes_written(self) -> u64 {
        self.output_bytes_written
    }

    /// Returns ordered chunk occurrences processed.
    #[must_use]
    pub const fn chunk_instances(self) -> u64 {
        self.chunk_instances
    }

    /// Returns distinct chunk identities reused from visible state.
    #[must_use]
    pub const fn reused_unique_chunks(self) -> u64 {
        self.reused_unique_chunks
    }

    /// Returns semantic operations completed by the sample.
    #[must_use]
    pub const fn operation_count(self) -> u64 {
        self.operation_count
    }
}

fn checked_add(target: &mut u64, incoming: u64, metric: &'static str) -> Result<(), ScenarioError> {
    let current = *target;
    *target = current
        .checked_add(incoming)
        .ok_or(ScenarioError::MetricOverflow {
            metric,
            current,
            incoming,
        })?;
    Ok(())
}
