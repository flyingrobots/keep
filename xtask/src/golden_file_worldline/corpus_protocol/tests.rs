//! This module owns corpus framing, table-boundary, and path grammar tests.

use std::path::PathBuf;

use super::{Corpus, GoldenError, protocol_source_path, table_rows};

#[path = "tests/malformed_corpus.rs"]
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
        let result = protocol_source_path(parameter);
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

#[cfg(unix)]
#[test]
fn opened_corpus_sources_survive_path_replacement() -> Result<(), GoldenError> {
    use std::env;
    use std::fs;
    use std::os::unix::fs::symlink;
    use std::process;

    let root = env::temp_dir().join(format!("keep-corpus-source-race-{}", process::id()));
    fs::create_dir(&root)
        .map_err(|source| GoldenError::io("create source test corpus", &root, source))?;
    let source_path = root.join("source.txt");
    let retained_path = root.join("retained.txt");
    let outside_path = root.with_extension("outside");
    fs::write(&source_path, b"admitted")
        .map_err(|source| GoldenError::io("write admitted source", &source_path, source))?;
    fs::write(&outside_path, b"substituted")
        .map_err(|source| GoldenError::io("write substituted source", &outside_path, source))?;

    let corpus = Corpus::open(root.clone())?;
    let source = corpus.source_file("source.txt")?;
    fs::rename(&source_path, &retained_path)
        .map_err(|source| GoldenError::io("retain admitted source", &source_path, source))?;
    symlink(&outside_path, &source_path)
        .map_err(|source| GoldenError::io("replace admitted source", &source_path, source))?;
    let observed = source.bounded_bytes(super::MAX_SOURCE_BYTES, "source-race")?;

    fs::remove_dir_all(&root)
        .map_err(|source| GoldenError::io("remove source test corpus", &root, source))?;
    fs::remove_file(&outside_path)
        .map_err(|source| GoldenError::io("remove substituted source", &outside_path, source))?;
    assert_eq!(observed, b"admitted");
    Ok(())
}

fn rows(lines: &[&str]) -> Result<Vec<super::TableRow>, GoldenError> {
    table_rows(
        "cases.tsv",
        "# keep.cases/v1",
        &["case"],
        lines.iter().map(|line| (*line).to_owned()).collect(),
    )
}
