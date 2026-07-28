//! Prepared-state dispatch for integrated benchmark workloads.

use keep::{AdmittedLayout, BlobId, ByteRange, ReferenceStore};

use crate::{
    BenchmarkCorpus, Scenario, ScenarioError, ScenarioObservation, VerificationPosture,
    scenario_ingest, scenario_ranges, scenario_read,
};

/// Prepared state for repeatable execution of one scenario.
pub struct PreparedScenario<'a> {
    scenario: Scenario,
    corpus: &'a BenchmarkCorpus,
    state: PreparedState,
}

enum PreparedState {
    None,
    Warm(ReferenceStore),
    Read {
        store: ReferenceStore,
        target: BlobId,
        layout: AdmittedLayout,
        ranges: Box<[ByteRange]>,
    },
    Verify {
        store: ReferenceStore,
        target: BlobId,
        layout: AdmittedLayout,
    },
}

impl<'a> PreparedScenario<'a> {
    /// Prepares state excluded from timed execution where the scenario calls
    /// for an already-populated store.
    ///
    /// # Errors
    ///
    /// Returns [`ScenarioError`] if fixed setup bytes cannot be ingested,
    /// published, ranged, or allocated.
    pub fn new(scenario: Scenario, corpus: &'a BenchmarkCorpus) -> Result<Self, ScenarioError> {
        let state = match scenario {
            Scenario::WarmIngest => {
                PreparedState::Warm(scenario_ingest::store_with(corpus.large_text(), scenario)?)
            }
            Scenario::SequentialRangeReads | Scenario::RandomRangeReads => {
                let (store, target, layout) =
                    scenario_ingest::published_store(corpus.large_binary(), scenario)?;
                let ranges = scenario_ranges::ranges(scenario, corpus.large_binary().len())?;
                PreparedState::Read {
                    store,
                    target,
                    layout,
                    ranges,
                }
            }
            Scenario::Verification => {
                let (store, target, layout) =
                    scenario_ingest::published_store(corpus.large_binary(), scenario)?;
                PreparedState::Verify {
                    store,
                    target,
                    layout,
                }
            }
            Scenario::ColdIngest
            | Scenario::NearNeighborEdits
            | Scenario::EarlyInsertion
            | Scenario::EarlyDeletion
            | Scenario::ManyTinyBlobs
            | Scenario::LargeBinary
            | Scenario::HighDeduplication
            | Scenario::ZeroDeduplication
            | Scenario::VariedInputPartitioning => PreparedState::None,
        };
        Ok(Self {
            scenario,
            corpus,
            state,
        })
    }

    /// Executes one exact sample of the prepared scenario.
    ///
    /// # Errors
    ///
    /// Returns [`ScenarioError`] for any ingestion, publication, verification,
    /// output, range, allocation, or checked-accounting refusal.
    pub fn run(&mut self) -> Result<ScenarioObservation, ScenarioError> {
        match &mut self.state {
            PreparedState::None => scenario_ingest::run(self.scenario, self.corpus),
            PreparedState::Warm(store) => {
                scenario_ingest::run_warm(self.scenario, self.corpus.large_text(), store)
            }
            PreparedState::Read {
                store,
                target,
                layout,
                ranges,
            } => scenario_read::run_ranges(self.scenario, store, *target, layout, ranges),
            PreparedState::Verify {
                store,
                target,
                layout,
            } => scenario_read::run_verification(self.scenario, store, *target, layout),
        }
    }

    /// Returns the stable scenario coordinate.
    #[must_use]
    pub const fn scenario(&self) -> Scenario {
        self.scenario
    }

    /// Returns the authentication law for the timed operation.
    #[must_use]
    pub const fn verification(&self) -> VerificationPosture {
        match self.scenario {
            Scenario::SequentialRangeReads | Scenario::RandomRangeReads => {
                VerificationPosture::SelectedChunks
            }
            Scenario::Verification => VerificationPosture::CompleteBlob,
            Scenario::ColdIngest
            | Scenario::WarmIngest
            | Scenario::NearNeighborEdits
            | Scenario::EarlyInsertion
            | Scenario::EarlyDeletion
            | Scenario::ManyTinyBlobs
            | Scenario::LargeBinary
            | Scenario::HighDeduplication
            | Scenario::ZeroDeduplication
            | Scenario::VariedInputPartitioning => VerificationPosture::IngestIdentity,
        }
    }
}
