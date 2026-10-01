//! Laws for integrated streaming CAS benchmark scenarios.

use std::error::Error;

use crate::{BenchmarkCorpus, PreparedScenario, Scenario};

const TIMED_RANGE_SOURCE: &str = include_str!("scenario_read.rs");

#[test]
fn scenario_catalog_covers_every_required_workload() {
    assert_eq!(
        Scenario::ALL.map(Scenario::name),
        [
            "cold-ingest",
            "warm-ingest",
            "repeated-near-neighbor-edits",
            "early-insertion",
            "early-deletion",
            "many-tiny-blobs",
            "large-binary",
            "high-deduplication",
            "zero-deduplication",
            "sequential-range-reads",
            "random-range-reads",
            "whole-blob-verification",
            "varied-input-partitioning",
        ]
    );
}

#[test]
fn every_scenario_executes_with_deterministic_semantic_metrics() -> Result<(), Box<dyn Error>> {
    let corpus = BenchmarkCorpus::generate()?;
    for scenario in Scenario::ALL {
        let first = PreparedScenario::new(scenario, &corpus)?.run()?;
        let second = PreparedScenario::new(scenario, &corpus)?.run()?;
        assert_eq!(first, second, "{}", scenario.name());
        assert!(first.operation_count() > 0);
        assert!(first.logical_bytes() > 0);
        assert!(first.verification().is_authenticated());
    }
    Ok(())
}

#[test]
fn scenario_metrics_preserve_reuse_and_verification_meaning() -> Result<(), Box<dyn Error>> {
    let corpus = BenchmarkCorpus::generate()?;
    let warm = PreparedScenario::new(Scenario::WarmIngest, &corpus)?.run()?;
    let high = PreparedScenario::new(Scenario::HighDeduplication, &corpus)?.run()?;
    let zero = PreparedScenario::new(Scenario::ZeroDeduplication, &corpus)?.run()?;
    let sequential = PreparedScenario::new(Scenario::SequentialRangeReads, &corpus)?.run()?;
    let random = PreparedScenario::new(Scenario::RandomRangeReads, &corpus)?.run()?;
    let verification = PreparedScenario::new(Scenario::Verification, &corpus)?.run()?;
    let partitioned = PreparedScenario::new(Scenario::VariedInputPartitioning, &corpus)?.run()?;

    assert_eq!(warm.materialized_bytes_written(), 0);
    assert_eq!(warm.authenticated_chunk_bytes_read(), warm.logical_bytes());
    assert!(warm.reused_unique_chunks() > 0);
    assert!(high.reused_unique_chunks() > 0);
    assert_eq!(zero.reused_unique_chunks(), 0);
    assert!(sequential.authenticated_chunk_bytes_read() >= sequential.output_bytes_written());
    assert!(random.authenticated_chunk_bytes_read() >= random.output_bytes_written());
    assert_eq!(
        verification.authenticated_chunk_bytes_read(),
        verification.logical_bytes()
    );
    assert_eq!(partitioned.source_bytes_read(), partitioned.logical_bytes());
    Ok(())
}

#[test]
fn timed_range_execution_contains_no_accounting_plans() {
    assert!(!TIMED_RANGE_SOURCE.contains("authenticated_range_bytes"));
    assert!(!TIMED_RANGE_SOURCE.contains("selected_entry_count"));
    assert!(!TIMED_RANGE_SOURCE.contains("plan_range"));
}

#[test]
fn range_accounting_counts_complete_selected_chunks_once() -> Result<(), Box<dyn Error>> {
    use keep::{ByteLength, ByteOffset, ByteRange};

    let corpus = BenchmarkCorpus::generate()?;
    let scenario = Scenario::SequentialRangeReads;
    let (store, target, layout) =
        crate::scenario_ingest::published_store(corpus.large_binary(), scenario)?;
    let length = target.logical_length().get();
    let requests = [
        ByteRange::new(ByteOffset::new(0), ByteLength::new(1))?,
        ByteRange::new(ByteOffset::new(0), ByteLength::new(length))?,
        ByteRange::new(ByteOffset::new(length), ByteLength::new(0))?,
    ];
    let ranges = crate::scenario_range_metrics::prepare(scenario, &layout, &requests)?;
    let observation = crate::scenario_read::run_ranges(scenario, &store, target, &ranges)?;
    let first_chunk_length = u64::from(
        layout
            .entries()
            .first()
            .ok_or("first chunk missing")?
            .chunk_id()
            .length()
            .get(),
    );
    assert_eq!(
        observation.authenticated_chunk_bytes_read(),
        length
            .checked_add(first_chunk_length)
            .ok_or("accounting overflow")?
    );
    assert_eq!(
        observation.output_bytes_written(),
        length.checked_add(1).ok_or("output overflow")?
    );
    Ok(())
}
