//! Laws for benchmark metadata policies and consistent sample evidence.

use super::{BenchmarkBaselineError, artifact, environment, report};

#[test]
fn optimized_metadata_policies_refuse_substitution_exactly() {
    let environment = environment();
    let valid = report(&environment, 13, 5);
    for (key, expected, observed) in [
        ("cpu-clock", "process", "wall"),
        ("peak-memory", "incremental-live-heap", "unknown"),
        ("verification", "mandatory", "disabled"),
        ("timing-unit", "nanoseconds", "milliseconds"),
        ("byte-unit", "bytes", "kilobytes"),
        ("ratio-encoding", "exact-numerator-denominator", "float"),
        ("sample-count", "100", "0"),
        ("sample-count", "100", "99"),
        ("sample-count", "100", "1001"),
        ("warmup-count", "5", "0"),
        ("warmup-count", "5", "101"),
    ] {
        let malformed = valid.replace(
            &format!("metadata\t{key}\t{expected}\n"),
            &format!("metadata\t{key}\t{observed}\n"),
        );
        assert!(
            matches!(artifact::validate(malformed.as_bytes(), &environment),
            Err(BenchmarkBaselineError::InvalidReportRow {
                expected: actual_expected, observed: actual_observed,
            }) if actual_expected == expected && actual_observed == observed)
        );
    }
}

#[test]
fn every_catalog_row_binds_its_sample_count_to_the_publication_policy() {
    let environment = environment();
    let valid = report(&environment, 13, 5);
    for row in valid
        .lines()
        .filter(|line| line.starts_with("scenario\t") || line.starts_with("profile\t"))
    {
        let malformed_row = row.replacen("\t100\t", "\t99\t", 1);
        let malformed = valid.replace(row, &malformed_row);
        assert!(
            matches!(artifact::validate(malformed.as_bytes(), &environment),
            Err(BenchmarkBaselineError::InvalidReportRow {
                expected: "100", observed,
            }) if observed == "99")
        );
    }
}
