//! Laws for admission of producer-representable benchmark counters.

use super::super::metric_error::ReportMetricError;
use super::{BenchmarkBaselineError, artifact, environment, report};

#[test]
fn oversized_counters_refuse_with_their_exact_metric_and_bound() {
    let environment = environment();
    let valid = report(&environment, 13, 5);
    for (name, metric) in [
        ("cold-ingest", "source-bytes-read"),
        ("cold-ingest", "output-bytes-written"),
        ("cold-ingest", "operation-count"),
        ("cold-ingest", "total-allocation-count"),
        ("cold-ingest", "total-allocated-bytes"),
        ("cold-ingest", "peak-live-allocation-count"),
        ("cold-ingest", "peak-live-heap-bytes"),
        ("fixed-64", "base-materialized-bytes"),
        ("fixed-64", "total-allocation-count"),
        ("fixed-64", "total-allocated-bytes"),
        ("fixed-64", "peak-live-heap-bytes"),
    ] {
        let malformed =
            super::metric_relation_tests::mutate(&valid, name, metric, "18446744073709551616");
        assert!(
            matches!(artifact::validate(malformed.as_bytes(), &environment),
            Err(BenchmarkBaselineError::Metric(ReportMetricError::Bound {
                metric: actual_metric, maximum: 18_446_744_073_709_551_615,
                observed: 18_446_744_073_709_551_616,
            })) if actual_metric == metric)
        );
    }
}

#[test]
fn the_maximum_counter_value_remains_admissible() {
    let environment = environment();
    let valid = report(&environment, 13, 5);
    for (name, metric) in [
        ("cold-ingest", "source-bytes-read"),
        ("cold-ingest", "total-allocation-count"),
        ("fixed-64", "base-materialized-bytes"),
    ] {
        let maximum =
            super::metric_relation_tests::mutate(&valid, name, metric, "18446744073709551615");
        assert!(artifact::validate(maximum.as_bytes(), &environment).is_ok());
    }
}
