//! Laws for canonical metric decimal encodings.

use super::{BenchmarkBaselineError, artifact, environment, report};

#[test]
fn noncanonical_metric_decimals_refuse_exactly() {
    let environment = environment();
    let valid = report(&environment, 13, 5);
    let prefix = "scenario\tcold-ingest\tingest-chunk-and-blob-identity\t";
    for observed in ["", "0100", "+100", "-100", "1.0"] {
        let malformed = valid.replace(&format!("{prefix}100\t"), &format!("{prefix}{observed}\t"));
        assert!(
            matches!(artifact::validate(malformed.as_bytes(), &environment),
            Err(BenchmarkBaselineError::InvalidReportRow {
                expected: "canonical unsigned decimal", observed: actual,
            }) if actual == observed)
        );
    }
}
