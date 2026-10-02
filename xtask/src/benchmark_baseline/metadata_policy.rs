//! This module owns the fixed optimized subprocess measurement policy.

use super::BenchmarkBaselineError;

pub(super) const SAMPLE_COUNT: &str = "100";

pub(super) fn admit(key: &str, observed: &str) -> Result<(), BenchmarkBaselineError> {
    let expected = match key {
        "cpu-clock" => "process",
        "peak-memory" => "incremental-live-heap",
        "verification" => "mandatory",
        "timing-unit" => "nanoseconds",
        "byte-unit" => "bytes",
        "ratio-encoding" => "exact-numerator-denominator",
        "sample-count" => SAMPLE_COUNT,
        "warmup-count" => "5",
        _ => return Ok(()),
    };
    require(expected, observed)
}

pub(super) fn require(
    expected: &'static str,
    observed: &str,
) -> Result<(), BenchmarkBaselineError> {
    if expected == observed {
        Ok(())
    } else {
        Err(BenchmarkBaselineError::InvalidReportRow {
            expected,
            observed: observed.to_owned(),
        })
    }
}
