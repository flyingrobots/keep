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

#[test]
fn corpus_read_options_refuse_blocking_io() {
    let options = format!("{:?}", super::nonblocking_read_options());
    assert!(options.contains("read: true"));
    assert!(options.contains("nonblock: true"));
}

#[test]
fn golden_diagnostics_escape_terminal_controls() {
    let diagnostic = GoldenError::violation("first\nError: forged\rrewrite\u{1b}[31m").to_string();
    assert_eq!(
        diagnostic,
        "golden corpus check failed: first\\nError: forged\\rrewrite\\u{1b}[31m"
    );
    assert_eq!(diagnostic.lines().count(), 1);
}

#[test]
fn corpus_tables_refuse_non_regular_handles() -> Result<(), GoldenError> {
    use std::env;
    use std::fs;
    use std::process;

    let root = env::temp_dir().join(format!("keep-corpus-table-type-{}", process::id()));
    fs::create_dir(&root)
        .map_err(|source| GoldenError::io("create table test corpus", &root, source))?;
    let table_path = root.join("cases.tsv");
    fs::create_dir(&table_path)
        .map_err(|source| GoldenError::io("create non-regular table", &table_path, source))?;

    let result = Corpus::open(root.clone())?.rows("cases.tsv", "# keep.cases/v1", &["case"]);
    let refused = matches!(
        result,
        Err(GoldenError::Violation(ref message))
            if message == "corpus entry is not a regular file: cases.tsv"
    );
    fs::remove_dir_all(&root)
        .map_err(|source| GoldenError::io("remove table test corpus", &root, source))?;
    assert!(refused);
    Ok(())
}

#[cfg(unix)]
#[test]
fn corpus_tables_refuse_symlink_substitution() -> Result<(), GoldenError> {
    use std::env;
    use std::fs;
    use std::os::unix::fs::symlink;
    use std::process;

    let root = env::temp_dir().join(format!("keep-corpus-table-link-{}", process::id()));
    let outside = root.with_extension("outside");
    fs::create_dir(&root)
        .map_err(|source| GoldenError::io("create table test corpus", &root, source))?;
    fs::write(&outside, "# keep.cases/v1\ncase\nsubstituted\n")
        .map_err(|source| GoldenError::io("write substituted table", &outside, source))?;
    let table_path = root.join("cases.tsv");
    symlink(&outside, &table_path)
        .map_err(|source| GoldenError::io("link substituted table", &table_path, source))?;

    let result = Corpus::open(root.clone())?.rows("cases.tsv", "# keep.cases/v1", &["case"]);
    let refused = matches!(
        result,
        Err(GoldenError::Io {
            action: "open corpus table",
            ref path,
            source: _,
        }) if path == &table_path
    );
    fs::remove_dir_all(&root)
        .map_err(|source| GoldenError::io("remove table test corpus", &root, source))?;
    fs::remove_file(&outside)
        .map_err(|source| GoldenError::io("remove substituted table", &outside, source))?;

    assert!(refused);
    Ok(())
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
