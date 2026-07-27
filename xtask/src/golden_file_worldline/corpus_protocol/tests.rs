//! This module owns corpus framing, table-boundary, and path grammar tests.

use std::path::PathBuf;

use super::{Corpus, GoldenError, protocol_source_path, table_rows};

mod malformed_corpus;

#[test]
fn tables_require_at_least_one_data_row() {
    let result = rows(&["# keep.cases/v1", "case"]);
    assert!(matches!(
        result,
        Err(GoldenError::Violation(ref message)) if message == "cases.tsv: table has no data rows"
    ));
}

#[test]
fn tables_refuse_zero_lines() {
    let result = rows(&[]);
    assert!(matches!(
        result,
        Err(GoldenError::Violation(ref message))
            if message == "cases.tsv: unsupported schema or empty table"
    ));
}

#[test]
fn tables_refuse_a_schema_without_columns() {
    let result = rows(&["# keep.cases/v1"]);
    assert!(matches!(
        result,
        Err(GoldenError::Violation(ref message))
            if message == "cases.tsv: unsupported schema or empty table"
    ));
}

#[test]
fn tables_admit_a_schema_columns_and_one_row() {
    let result = rows(&["# keep.cases/v1", "case", "example"]);
    assert!(matches!(result, Ok(ref table_rows) if table_rows.len() == 1));
}

#[test]
fn source_paths_refuse_host_dependent_or_noncanonical_spellings() {
    let corpus = Corpus::new(PathBuf::from("unused-corpus-root"));
    for parameter in [
        "",
        ".",
        "..",
        "/absolute",
        "C:/drive-prefix",
        r"inputs\host-separator",
        "inputs//empty-segment",
        "inputs/./dot-segment",
        "inputs/../parent-segment",
    ] {
        let result = corpus.source_path(parameter);
        assert!(matches!(
            result,
            Err(GoldenError::Violation(ref message))
                if message == &format!("unsafe source path: {parameter}")
        ));
    }
}

#[test]
fn source_paths_admit_canonical_posix_segments() {
    let result = protocol_source_path("inputs/small-text.txt");
    assert_eq!(result.ok(), Some(PathBuf::from("inputs/small-text.txt")));
}

fn rows(lines: &[&str]) -> Result<Vec<super::TableRow>, GoldenError> {
    table_rows(
        "cases.tsv",
        "# keep.cases/v1",
        &["case"],
        lines.iter().map(|line| (*line).to_owned()).collect(),
    )
}
