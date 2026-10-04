//! Model laws for frozen report row order and required catalogs.

use super::{BenchmarkBaselineError, artifact, environment, report};

#[test]
fn every_adjacent_row_transposition_refuses_at_the_first_order_violation()
-> Result<(), Box<dyn std::error::Error>> {
    let environment = environment();
    let valid = report(&environment, 13, 5);
    let rows: Vec<_> = valid.lines().collect();
    for (index, pair) in rows.windows(2).enumerate() {
        let [first, second] = pair else {
            return Err("invalid window fixture".into());
        };
        let successor = index.checked_add(1).ok_or("row index overflow")?;
        let mut malformed = String::new();
        for (position, row) in rows.iter().enumerate() {
            malformed.push_str(if position == index {
                second
            } else if position == successor {
                first
            } else {
                row
            });
            malformed.push('\n');
        }
        let error = artifact::validate(malformed.as_bytes(), &environment)
            .err()
            .ok_or("transposed rows were admitted")?;
        if first.starts_with("schema\t") {
            assert!(matches!(
                error,
                BenchmarkBaselineError::ReportViolation {
                    reason: "report-schema"
                }
            ));
        } else {
            let expected = expected_row(first)?;
            assert!(matches!(error, BenchmarkBaselineError::InvalidReportRow {
                expected: actual_expected, observed,
            } if actual_expected == expected && observed == *second));
        }
    }
    Ok(())
}

#[test]
fn deleting_any_catalog_row_refuses_the_exact_missing_catalog() {
    let environment = environment();
    let valid = report(&environment, 13, 5);
    for row in valid
        .lines()
        .filter(|line| line.starts_with("scenario\t") || line.starts_with("profile\t"))
    {
        let malformed = valid.replace(&format!("{row}\n"), "");
        let expected = if row.starts_with("scenario\t") {
            "report-scenario-count"
        } else {
            "report-profile-count"
        };
        assert!(
            matches!(artifact::validate(malformed.as_bytes(), &environment),
            Err(BenchmarkBaselineError::ReportViolation { reason }) if reason == expected)
        );
    }
}

fn expected_row(row: &str) -> Result<String, Box<dyn std::error::Error>> {
    if row.starts_with("metadata\t") {
        return row
            .split('\t')
            .nth(1)
            .map(str::to_owned)
            .ok_or_else(|| "missing metadata key".into());
    }
    if row.starts_with("scenario\t") {
        return Ok(row.split('\t').take(3).collect::<Vec<_>>().join("\t"));
    }
    if row.starts_with("profile\t") {
        return Ok(row.split('\t').take(7).collect::<Vec<_>>().join("\t"));
    }
    Ok(row.to_owned())
}
