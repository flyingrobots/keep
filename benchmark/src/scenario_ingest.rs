//! Streaming ingestion scenario sequencing.

use keep::{AdmittedLayout, BlobId, LayoutEntryLimit, ReferenceStore};

use crate::metered_reader::MeteredReader;
use crate::scenario_ingest_operation::{DEFAULT_PARTITION, TINY_PARTITION, ingest, new_store};
use crate::{BenchmarkCorpus, Scenario, ScenarioError, ScenarioObservation, VerificationPosture};

pub(super) fn run(
    scenario: Scenario,
    corpus: &BenchmarkCorpus,
) -> Result<ScenarioObservation, ScenarioError> {
    match scenario {
        Scenario::ColdIngest => run_sequence(scenario, &[corpus.large_text()]),
        Scenario::NearNeighborEdits => run_sequence(
            scenario,
            &[
                corpus.edit_base(),
                corpus.near_neighbor(),
                corpus.edit_base(),
                corpus.near_neighbor(),
            ],
        ),
        Scenario::EarlyInsertion => {
            run_sequence(scenario, &[corpus.edit_base(), corpus.early_insertion()])
        }
        Scenario::EarlyDeletion => {
            run_sequence(scenario, &[corpus.edit_base(), corpus.early_deletion()])
        }
        Scenario::ManyTinyBlobs => run_tiny(scenario, corpus.tiny_blobs()),
        Scenario::LargeBinary => run_sequence(scenario, &[corpus.large_binary()]),
        Scenario::HighDeduplication => {
            run_sequence(scenario, &[corpus.edit_base(), corpus.edit_base()])
        }
        Scenario::ZeroDeduplication => {
            run_sequence(scenario, &[corpus.large_binary(), corpus.zero_dedup()])
        }
        Scenario::VariedInputPartitioning => run_partitions(scenario, corpus.large_binary()),
        Scenario::WarmIngest
        | Scenario::SequentialRangeReads
        | Scenario::RandomRangeReads
        | Scenario::Verification => Err(ScenarioError::CorpusRangeUnavailable {
            target: "prepared-scenario-dispatch",
            available: 0,
        }),
    }
}

pub(super) fn run_warm(
    scenario: Scenario,
    source: &[u8],
    store: &mut ReferenceStore,
) -> Result<ScenarioObservation, ScenarioError> {
    let mut observation = ScenarioObservation::new(scenario, VerificationPosture::IngestIdentity);
    ingest(store, scenario, source, DEFAULT_PARTITION, &mut observation)?;
    Ok(observation)
}

pub(super) fn store_with(
    source: &[u8],
    scenario: Scenario,
) -> Result<ReferenceStore, ScenarioError> {
    let (store, _target, _layout) = published_store(source, scenario)?;
    Ok(store)
}

pub(super) fn published_store(
    source: &[u8],
    scenario: Scenario,
) -> Result<(ReferenceStore, BlobId, AdmittedLayout), ScenarioError> {
    let mut store = new_store();
    let mut reader = MeteredReader::new(source, DEFAULT_PARTITION)?;
    let staged = store
        .stage(&mut reader, LayoutEntryLimit::MAXIMUM)
        .map_err(|source| ScenarioError::Ingestion {
            scenario,
            source: Box::new(source),
        })?;
    let target = staged.target();
    let layout = staged.layout().clone();
    let _published = staged
        .commit(&mut store)
        .map_err(|source| ScenarioError::Publication {
            scenario,
            source: Box::new(source),
        })?;
    Ok((store, target, layout))
}

fn run_sequence(
    scenario: Scenario,
    sources: &[&[u8]],
) -> Result<ScenarioObservation, ScenarioError> {
    let mut store = new_store();
    let mut observation = ScenarioObservation::new(scenario, VerificationPosture::IngestIdentity);
    for source in sources {
        ingest(
            &mut store,
            scenario,
            source,
            DEFAULT_PARTITION,
            &mut observation,
        )?;
    }
    Ok(observation)
}

fn run_tiny(
    scenario: Scenario,
    sources: &[Box<[u8]>],
) -> Result<ScenarioObservation, ScenarioError> {
    let mut store = new_store();
    let mut observation = ScenarioObservation::new(scenario, VerificationPosture::IngestIdentity);
    for source in sources {
        ingest(
            &mut store,
            scenario,
            source,
            TINY_PARTITION,
            &mut observation,
        )?;
    }
    Ok(observation)
}

fn run_partitions(scenario: Scenario, source: &[u8]) -> Result<ScenarioObservation, ScenarioError> {
    const PARTITIONS: [&[usize]; 4] = [&[8_192], &[1], &[7, 1_024, 3, 8_192], &[4_096, 31]];
    let source = source
        .get(..65_536)
        .ok_or(ScenarioError::CorpusRangeUnavailable {
            target: "partitioned-input-prefix",
            available: source.len(),
        })?;
    let mut observation = ScenarioObservation::new(scenario, VerificationPosture::IngestIdentity);
    for widths in PARTITIONS {
        let mut store = new_store();
        ingest(&mut store, scenario, source, widths, &mut observation)?;
    }
    Ok(observation)
}
