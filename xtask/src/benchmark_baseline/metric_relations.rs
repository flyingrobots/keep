//! This module owns arithmetic relationships between admitted metric fields.

use super::BenchmarkBaselineError;
use super::metric_error::ReportMetricError;

pub(super) fn scenario(values: &str) -> Result<(), BenchmarkBaselineError> {
    let [
        samples,
        logical,
        read,
        written,
        _source,
        _output,
        read_num,
        read_den,
        write_num,
        write_den,
        dedup_num,
        dedup_den,
        reused,
        chunks,
        _operations,
        rate,
        wall_total,
        wall50,
        wall95,
        wall99,
        cpu_total,
        cpu50,
        cpu95,
        cpu99,
        _allocations,
        _allocated,
        _peak_count,
        _peak_heap,
    ] = numbers::<28>(values)?;
    for (metric, expected, observed) in [
        ("read-amplification-numerator", read, read_num),
        ("read-amplification-denominator", logical, read_den),
        ("write-amplification-numerator", written, write_num),
        ("write-amplification-denominator", logical, write_den),
        ("deduplication-ratio-numerator", logical, dedup_num),
        ("deduplication-ratio-denominator", written, dedup_den),
    ] {
        equal(metric, expected, observed)?;
    }
    at_most("reused-unique-chunks", chunks, reused)?;
    percentiles("wall-percentiles", [wall50, wall95, wall99, wall_total])?;
    percentiles("cpu-percentiles", [cpu50, cpu95, cpu99, cpu_total])?;
    throughput(logical, samples, wall_total, rate)
}

pub(super) fn profile(values: &str) -> Result<(), BenchmarkBaselineError> {
    let [
        samples,
        rate,
        wall_total,
        wall50,
        wall95,
        wall99,
        _cpu_total,
        _allocations,
        _allocated,
        _peak_heap,
        base_chunks,
        _base_bytes,
        inserted,
        deleted,
        neighbor,
    ] = numbers::<15>(values)?;
    for (metric, reused) in [
        ("insertion-reused-chunks", inserted),
        ("deletion-reused-chunks", deleted),
        ("neighbor-reused-chunks", neighbor),
    ] {
        at_most(metric, base_chunks, reused)?;
    }
    percentiles(
        "profile-wall-percentiles",
        [wall50, wall95, wall99, wall_total],
    )?;
    // The frozen v1 catalog's timed large-text member is exactly one MiB.
    throughput(1_048_576, samples, wall_total, rate)
}

fn numbers<const N: usize>(values: &str) -> Result<[u128; N], BenchmarkBaselineError> {
    let mut fields = values.split('\t');
    let mut admitted = [0; N];
    for slot in &mut admitted {
        let observed = fields.next().unwrap_or_default();
        *slot = observed.parse().map_err(|source| {
            BenchmarkBaselineError::Metric(ReportMetricError::Encoding {
                observed: observed.to_owned(),
                source,
            })
        })?;
    }
    if fields.next().is_some() {
        return Err(BenchmarkBaselineError::InvalidReportRow {
            expected: "complete metric row",
            observed: values.to_owned(),
        });
    }
    Ok(admitted)
}

const fn equal(
    metric: &'static str,
    expected: u128,
    observed: u128,
) -> Result<(), BenchmarkBaselineError> {
    if expected == observed {
        Ok(())
    } else {
        Err(BenchmarkBaselineError::Metric(
            ReportMetricError::Mismatch {
                metric,
                expected,
                observed,
            },
        ))
    }
}

const fn at_most(
    metric: &'static str,
    maximum: u128,
    observed: u128,
) -> Result<(), BenchmarkBaselineError> {
    if observed <= maximum {
        Ok(())
    } else {
        Err(BenchmarkBaselineError::Metric(ReportMetricError::Bound {
            metric,
            maximum,
            observed,
        }))
    }
}

fn percentiles(metric: &'static str, values: [u128; 4]) -> Result<(), BenchmarkBaselineError> {
    let [p50, p95, p99, total] = values;
    for (maximum, observed) in [(p95, p50), (p99, p95), (total, p99)] {
        at_most(metric, maximum, observed)?;
    }
    Ok(())
}

fn throughput(
    bytes: u128,
    samples: u128,
    duration: u128,
    observed: u128,
) -> Result<(), BenchmarkBaselineError> {
    let expected = bytes
        .checked_mul(samples)
        .and_then(|value| value.checked_mul(1_000_000_000))
        .and_then(|value| value.checked_div(duration))
        .ok_or(BenchmarkBaselineError::Metric(
            ReportMetricError::Arithmetic {
                bytes,
                samples,
                duration,
            },
        ))?;
    equal("logical-bytes-per-second", expected, observed)
}

#[cfg(test)]
mod tests {
    use super::{BenchmarkBaselineError, ReportMetricError};

    #[test]
    fn throughput_product_overflow_refuses_instead_of_saturating() {
        assert!(matches!(
            super::throughput(u128::MAX, 100, 1, 0),
            Err(BenchmarkBaselineError::Metric(
                ReportMetricError::Arithmetic {
                    bytes: u128::MAX,
                    samples: 100,
                    duration: 1,
                }
            ))
        ));
    }
}
