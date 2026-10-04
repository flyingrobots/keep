//! Model laws for required metadata and complete metric row widths.

use super::{BenchmarkBaselineError, artifact, environment, report};

#[test]
fn missing_and_unknown_metadata_refuse_the_exact_required_coordinate()
-> Result<(), Box<dyn std::error::Error>> {
    let environment = environment();
    let valid = report(&environment, 13, 5);
    for row in valid.lines().filter(|line| line.starts_with("metadata\t")) {
        let key = row.split('\t').nth(1).ok_or("missing fixture key")?;
        let unknown = row.replacen(&format!("metadata\t{key}\t"), "metadata\tunknown\t", 1);
        let successor = valid
            .lines()
            .skip_while(|line| *line != row)
            .nth(1)
            .ok_or("missing successor fixture")?;
        for (replacement, observed) in [
            (String::new(), successor),
            (format!("{unknown}\n"), unknown.as_str()),
        ] {
            let malformed = valid.replacen(&format!("{row}\n"), &replacement, 1);
            let error = artifact::validate(malformed.as_bytes(), &environment)
                .err()
                .ok_or("incomplete metadata was admitted")?;
            exact_metadata_refusal(error, key, observed);
        }
    }
    Ok(())
}

fn exact_metadata_refusal(error: BenchmarkBaselineError, key: &str, observed: &str) {
    if [
        "build-profile",
        "git-commit",
        "git-tree",
        "rustc-version",
        "target-triple",
        "os-description",
        "cpu-model",
        "logical-cpu-count",
    ]
    .contains(&key)
    {
        let expected = format!("report-{key}");
        assert!(
            matches!(error, BenchmarkBaselineError::ReportViolation { reason } if reason == expected)
        );
    } else {
        assert!(matches!(error, BenchmarkBaselineError::InvalidReportRow {
            expected, observed: actual,
        } if expected == key && actual == observed));
    }
}

#[test]
fn every_metric_row_refuses_missing_or_extra_fields_exactly()
-> Result<(), Box<dyn std::error::Error>> {
    let environment = environment();
    let valid = report(&environment, 13, 5);
    for row in valid
        .lines()
        .filter(|line| line.starts_with("scenario\t") || line.starts_with("profile\t"))
    {
        let (truncated, _field) = row.rsplit_once('\t').ok_or("missing metric fixture")?;
        let extended = format!("{row}\t1");
        for observed in [truncated, extended.as_str()] {
            let malformed = valid.replacen(&format!("{row}\n"), &format!("{observed}\n"), 1);
            assert!(
                matches!(artifact::validate(malformed.as_bytes(), &environment),
                Err(BenchmarkBaselineError::InvalidReportRow {
                    expected: "complete metric row", observed: actual,
                }) if actual == observed)
            );
        }
    }
    Ok(())
}
