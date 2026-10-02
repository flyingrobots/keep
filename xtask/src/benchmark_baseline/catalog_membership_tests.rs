//! Model laws for exact closed catalog membership without replacement aliases.

use super::{BenchmarkBaselineError, artifact, environment, report};

#[test]
fn unknown_and_duplicate_catalog_members_refuse_at_the_exact_slot()
-> Result<(), Box<dyn std::error::Error>> {
    let environment = environment();
    let valid = report(&environment, 13, 5);
    for row in valid
        .lines()
        .filter(|line| line.starts_with("scenario\t") || line.starts_with("profile\t"))
    {
        let mut fields = row.split('\t');
        let kind = fields.next().ok_or("missing catalog kind")?;
        let name = fields.next().ok_or("missing catalog name")?;
        let width = if kind == "scenario" { 3 } else { 7 };
        let expected = row.split('\t').take(width).collect::<Vec<_>>().join("\t");
        let unknown = row.replacen(
            &format!("{kind}\t{name}\t"),
            &format!("{kind}\tunknown\t"),
            1,
        );
        let duplicate = valid
            .lines()
            .find(|candidate| candidate.starts_with(&format!("{kind}\t")) && *candidate != row)
            .ok_or("missing duplicate fixture")?;
        for observed in [unknown.as_str(), duplicate] {
            let malformed = valid.replacen(&format!("{row}\n"), &format!("{observed}\n"), 1);
            assert!(
                matches!(artifact::validate(malformed.as_bytes(), &environment),
                Err(BenchmarkBaselineError::InvalidReportRow {
                    expected: actual_expected, observed: actual_observed,
                }) if actual_expected == expected && actual_observed == observed)
            );
        }
    }
    Ok(())
}
