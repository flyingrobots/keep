//! Named streaming CAS workload scenarios and semantic observations.

/// Reproducible integrated workloads required by the benchmark baseline.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Scenario {
    /// Cold source-like ingest into a new store.
    ColdIngest,
    /// Identical source-like ingest into a populated store.
    WarmIngest,
    /// Repeated nearby source substitutions.
    NearNeighborEdits,
    /// Early source insertion after a committed base.
    EarlyInsertion,
    /// Early source deletion after a committed base.
    EarlyDeletion,
    /// Many independently identified tiny blobs.
    ManyTinyBlobs,
    /// One large opaque binary blob.
    LargeBinary,
    /// Exact duplicate ingestion with maximal reuse.
    HighDeduplication,
    /// Independent opaque inputs with no intentional reuse.
    ZeroDeduplication,
    /// Ordered ranges covering one committed blob.
    SequentialRangeReads,
    /// Deterministically permuted ranges within one committed blob.
    RandomRangeReads,
    /// Authenticated whole-blob reconstruction.
    Verification,
    /// Equivalent ingestion under varied source read partitions.
    VariedInputPartitioning,
}

impl Scenario {
    /// Complete stable workload order.
    pub const ALL: [Self; 13] = [
        Self::ColdIngest,
        Self::WarmIngest,
        Self::NearNeighborEdits,
        Self::EarlyInsertion,
        Self::EarlyDeletion,
        Self::ManyTinyBlobs,
        Self::LargeBinary,
        Self::HighDeduplication,
        Self::ZeroDeduplication,
        Self::SequentialRangeReads,
        Self::RandomRangeReads,
        Self::Verification,
        Self::VariedInputPartitioning,
    ];

    /// Returns the stable scenario coordinate.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::ColdIngest => "cold-ingest",
            Self::WarmIngest => "warm-ingest",
            Self::NearNeighborEdits => "repeated-near-neighbor-edits",
            Self::EarlyInsertion => "early-insertion",
            Self::EarlyDeletion => "early-deletion",
            Self::ManyTinyBlobs => "many-tiny-blobs",
            Self::LargeBinary => "large-binary",
            Self::HighDeduplication => "high-deduplication",
            Self::ZeroDeduplication => "zero-deduplication",
            Self::SequentialRangeReads => "sequential-range-reads",
            Self::RandomRangeReads => "random-range-reads",
            Self::Verification => "whole-blob-verification",
            Self::VariedInputPartitioning => "varied-input-partitioning",
        }
    }
}

/// Authentication law exercised by one scenario.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VerificationPosture {
    /// Ingestion computes exact chunk and complete logical identities.
    IngestIdentity,
    /// A range read authenticates every complete selected chunk.
    SelectedChunks,
    /// Reconstruction authenticates chunks, boundaries, and complete identity.
    CompleteBlob,
}

impl VerificationPosture {
    /// Returns whether the workload retains an authentication law.
    ///
    /// The benchmark has no disabled posture; this method makes that invariant
    /// explicit to report and contract consumers.
    #[must_use]
    pub const fn is_authenticated(self) -> bool {
        true
    }

    /// Returns the stable report coordinate.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::IngestIdentity => "ingest-chunk-and-blob-identity",
            Self::SelectedChunks => "selected-complete-chunks",
            Self::CompleteBlob => "chunks-profile-and-blob",
        }
    }
}

pub use crate::scenario_execution::PreparedScenario;
pub use crate::scenario_observation::ScenarioObservation;

#[cfg(test)]
#[path = "scenario_tests.rs"]
mod tests;
