//! This module owns rejection of ambiguous report metadata coordinates.

use std::collections::BTreeSet;

use super::BenchmarkBaselineError;

pub(super) fn admit(report: &str) -> Result<(), BenchmarkBaselineError> {
    let mut coordinates = BTreeSet::new();
    for line in report.lines() {
        let mut fields = line.split('\t');
        if fields.next() != Some("metadata") {
            continue;
        }
        if let Some(coordinate) = fields.next()
            && !coordinates.insert(coordinate)
        {
            return Err(BenchmarkBaselineError::DuplicateReportMetadata {
                coordinate: coordinate.to_owned(),
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::BenchmarkBaselineError;

    #[test]
    fn conflicting_and_identical_metadata_repetitions_both_refuse() {
        for repeated in ["first", "second"] {
            let report = format!("metadata\tgit-commit\tfirst\nmetadata\tgit-commit\t{repeated}\n");
            assert!(matches!(
                super::admit(&report),
                Err(BenchmarkBaselineError::DuplicateReportMetadata { coordinate })
                    if coordinate == "git-commit"
            ));
        }
    }

    #[test]
    fn distinct_coordinates_may_share_values() {
        assert!(super::admit("metadata\tfirst\tvalue\nmetadata\tsecond\tvalue\n").is_ok());
    }
}
