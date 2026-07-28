//! Laws for integrated streaming CAS benchmark scenarios.

use std::error::Error;

use crate::{BenchmarkCorpus, PreparedScenario, Scenario};

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
    assert!(warm.reused_unique_chunks() > 0);
    assert!(high.reused_unique_chunks() > 0);
    assert_eq!(zero.reused_unique_chunks(), 0);
    assert!(sequential.authenticated_chunk_bytes_read() >= sequential.output_bytes_written());
    assert!(random.authenticated_chunk_bytes_read() >= random.output_bytes_written());
    assert_eq!(
        verification.authenticated_chunk_bytes_read(),
        verification
            .logical_bytes()
            .checked_mul(2)
            .ok_or("verification read overflow")?
    );
    assert_eq!(partitioned.source_bytes_read(), partitioned.logical_bytes());
    Ok(())
}
