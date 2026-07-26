use super::{GoldenError, table_rows};

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

fn rows(lines: &[&str]) -> Result<Vec<super::TableRow>, GoldenError> {
    table_rows(
        "cases.tsv",
        "# keep.cases/v1",
        &["case"],
        lines.iter().map(|line| (*line).to_owned()).collect(),
    )
}
