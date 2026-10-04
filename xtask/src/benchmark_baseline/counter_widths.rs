//! This module owns the portable u64 widths of v1 byte and count metrics.

use super::BenchmarkBaselineError;
use super::metric_error::ReportMetricError;
use super::report_schema::{PROFILE_HEADER, SCENARIO_HEADER};

const COUNTERS: &[&str] = &[
    "logical-bytes",
    "physical-bytes-read",
    "physical-bytes-written",
    "source-bytes-read",
    "output-bytes-written",
    "read-amplification-numerator",
    "read-amplification-denominator",
    "write-amplification-numerator",
    "write-amplification-denominator",
    "deduplication-ratio-numerator",
    "deduplication-ratio-denominator",
    "reused-unique-chunks",
    "chunk-instances",
    "operation-count",
    "total-allocation-count",
    "total-allocated-bytes",
    "peak-live-allocation-count",
    "peak-live-heap-bytes",
    "base-unique-chunks",
    "base-materialized-bytes",
    "insertion-reused-chunks",
    "deletion-reused-chunks",
    "neighbor-reused-chunks",
];

pub(super) fn scenario(values: &str) -> Result<(), BenchmarkBaselineError> {
    admit(SCENARIO_HEADER, values, 3)
}

pub(super) fn profile(values: &str) -> Result<(), BenchmarkBaselineError> {
    admit(PROFILE_HEADER, values, 7)
}

fn admit(header: &'static str, values: &str, prefix: usize) -> Result<(), BenchmarkBaselineError> {
    for (metric, value) in header.split('\t').skip(prefix).zip(values.split('\t')) {
        if !COUNTERS.contains(&metric) {
            continue;
        }
        let observed = value.parse::<u128>().map_err(|source| {
            BenchmarkBaselineError::Metric(ReportMetricError::Encoding {
                observed: value.to_owned(),
                source,
            })
        })?;
        if observed > u128::from(u64::MAX) {
            return Err(BenchmarkBaselineError::Metric(ReportMetricError::Bound {
                metric,
                maximum: u128::from(u64::MAX),
                observed,
            }));
        }
    }
    Ok(())
}
