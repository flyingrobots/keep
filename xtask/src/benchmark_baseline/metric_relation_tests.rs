//! Laws for admission of internally consistent benchmark evidence.

use super::super::metric_error::ReportMetricError;
use super::{BenchmarkBaselineError, artifact, environment, report};

#[test]
fn ratio_and_throughput_fields_bind_to_measured_counters() {
    let environment = environment();
    let valid = report(&environment, 13, 5);
    for (column, metric, expected) in [
        (
            "read-amplification-numerator",
            "read-amplification-numerator",
            0,
        ),
        (
            "read-amplification-denominator",
            "read-amplification-denominator",
            1_048_576,
        ),
        (
            "write-amplification-numerator",
            "write-amplification-numerator",
            1_048_576,
        ),
        (
            "write-amplification-denominator",
            "write-amplification-denominator",
            1_048_576,
        ),
        (
            "deduplication-ratio-numerator",
            "deduplication-ratio-numerator",
            1_048_576,
        ),
        (
            "deduplication-ratio-denominator",
            "deduplication-ratio-denominator",
            1_048_576,
        ),
        (
            "logical-bytes-per-second",
            "logical-bytes-per-second",
            166_506_928,
        ),
    ] {
        let malformed = mutate(&valid, "cold-ingest", column, "1");
        assert!(
            matches!(artifact::validate(malformed.as_bytes(), &environment),
            Err(BenchmarkBaselineError::Metric(ReportMetricError::Mismatch {
                metric: actual, expected: actual_expected, observed: 1,
            })) if actual == metric && actual_expected == expected)
        );
    }
}

#[test]
fn percentiles_and_reused_chunks_are_bounded_by_their_witnesses() {
    let environment = environment();
    let valid = report(&environment, 13, 5);
    for (name, column, metric, maximum) in [
        (
            "cold-ingest",
            "reused-unique-chunks",
            "reused-unique-chunks",
            13,
        ),
        (
            "cold-ingest",
            "p50-wall-time-ns",
            "wall-percentiles",
            6_503_458,
        ),
        (
            "cold-ingest",
            "p50-cpu-time-ns",
            "cpu-percentiles",
            6_449_000,
        ),
        (
            "fixed-64",
            "insertion-reused-chunks",
            "insertion-reused-chunks",
            32,
        ),
        (
            "fixed-64",
            "p50-wall-time-ns",
            "profile-wall-percentiles",
            1_373_875,
        ),
    ] {
        let malformed = mutate(&valid, name, column, "999999999999");
        assert!(
            matches!(artifact::validate(malformed.as_bytes(), &environment),
            Err(BenchmarkBaselineError::Metric(ReportMetricError::Bound {
                metric: actual, maximum: actual_maximum, observed: 999_999_999_999,
            })) if actual == metric && actual_maximum == maximum)
        );
    }
}

#[test]
fn throughput_refuses_zero_duration_without_arithmetic_approximation() {
    let environment = environment();
    let valid = report(&environment, 13, 5);
    let mut malformed = valid;
    for column in [
        "total-wall-time-ns",
        "p50-wall-time-ns",
        "p95-wall-time-ns",
        "p99-wall-time-ns",
    ] {
        malformed = mutate(&malformed, "cold-ingest", column, "0");
    }
    assert!(matches!(
        artifact::validate(malformed.as_bytes(), &environment),
        Err(BenchmarkBaselineError::Metric(
            ReportMetricError::Arithmetic {
                bytes: 1_048_576,
                samples: 100,
                duration: 0,
            }
        ))
    ));
}

#[test]
fn historical_metric_relationships_are_admissible() -> Result<(), BenchmarkBaselineError> {
    for row in super::HISTORICAL_BASELINE.lines() {
        if row.starts_with("scenario\t") {
            let values = row.split('\t').skip(3).collect::<Vec<_>>().join("\t");
            super::super::metric_relations::scenario(&values)?;
        }
        if row.starts_with("profile\t") {
            let values = row.split('\t').skip(7).collect::<Vec<_>>().join("\t");
            super::super::metric_relations::profile(&values)?;
        }
    }
    Ok(())
}

fn mutate(report: &str, name: &str, column: &str, value: &str) -> String {
    let mut output = String::new();
    let mut header = "";
    for row in report.lines() {
        if row.starts_with("scenario-header\t") || row.starts_with("profile-header\t") {
            header = row;
        }
        let mut fields = row.split('\t');
        let kind = fields.next();
        let row_name = fields.next();
        if matches!(kind, Some("scenario" | "profile")) && row_name == Some(name) {
            let changed = row
                .split('\t')
                .zip(header.split('\t'))
                .map(|(original, field)| if field == column { value } else { original })
                .collect::<Vec<_>>()
                .join("\t");
            output.push_str(&changed);
        } else {
            output.push_str(row);
        }
        output.push('\n');
    }
    output
}

#[test]
fn overflowing_metric_decoding_retains_the_original_parse_failure()
-> Result<(), Box<dyn std::error::Error>> {
    use std::error::Error;
    use std::num::{IntErrorKind, ParseIntError};

    let environment = environment();
    let valid = report(&environment, 13, 5);
    let overflow = "340282366920938463463374607431768211456";
    let malformed = mutate(&valid, "cold-ingest", "logical-bytes", overflow);
    let error = artifact::validate(malformed.as_bytes(), &environment)
        .err()
        .ok_or_else(|| std::io::Error::other("overflow was admitted"))?;
    assert!(
        matches!(&error, BenchmarkBaselineError::Metric(ReportMetricError::Encoding {
        observed, source,
    }) if observed == overflow && source.kind() == &IntErrorKind::PosOverflow)
    );
    assert_eq!(
        error
            .source()
            .and_then(Error::source)
            .and_then(|source| source.downcast_ref::<ParseIntError>())
            .map(ParseIntError::kind),
        Some(&IntErrorKind::PosOverflow)
    );
    Ok(())
}
