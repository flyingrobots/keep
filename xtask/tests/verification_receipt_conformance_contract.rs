//! Repository-shape evidence for the verification-receipt corpus.

#![cfg(feature = "repository-tasks")]

use std::collections::BTreeSet;
use std::ffi::OsString;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const CORPUS_ROOT: &str = "conformance/verification-receipt/v1";
const REQUIRED_PATHS: &[&str] = &[
    "README.md",
    "ORIGIN.md",
    "artifacts.tsv",
    "reference-complete-blob-report.hex",
    "durable-corrupt-chunk-refusal.hex",
    "reference-unsupported-framing-refusal.hex",
];

fn repository_root() -> Result<PathBuf, io::Error> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| io::Error::other("xtask manifest directory has no parent"))
}

#[test]
fn receipt_corpus_has_one_complete_regular_file_shape() -> Result<(), io::Error> {
    let root = repository_root()?.join(CORPUS_ROOT);
    let expected: BTreeSet<OsString> = REQUIRED_PATHS.iter().map(OsString::from).collect();
    let mut observed = BTreeSet::new();
    for entry in fs::read_dir(&root)? {
        let entry = entry?;
        assert!(
            entry.file_type()?.is_file(),
            "{} is not a regular file",
            entry.path().display()
        );
        observed.insert(entry.file_name());
    }
    assert_eq!(
        observed, expected,
        "verification-receipt corpus shape drifted"
    );
    Ok(())
}

#[test]
fn every_receipt_fixture_is_one_384_byte_record_in_the_artifact_table() -> Result<(), io::Error> {
    let root = repository_root()?.join(CORPUS_ROOT);
    let table = fs::read_to_string(root.join("artifacts.tsv"))?;
    let mut lines = table.lines();
    assert_eq!(lines.next(), Some("keep.verification-receipt.artifacts/v1"));
    assert_eq!(
        lines.next(),
        Some("case\tkind\tbyte_length\tchecksum_hex\tfixture")
    );
    let mut rows = 0_usize;
    for row in lines {
        let fields: Vec<&str> = row.split('\t').collect();
        assert_eq!(fields.len(), 5, "{row}");
        assert_eq!(fields.get(2), Some(&"384"), "{row}");
        assert!(
            matches!(fields.get(1), Some(&"report" | &"refusal")),
            "{row}"
        );
        let fixture = fs::read_to_string(root.join(fields.get(4).unwrap_or(&"")))?;
        assert_eq!(
            fixture.len(),
            769,
            "{row}: fixture is not 384 bytes plus LF"
        );
        assert!(fixture.ends_with('\n'), "{row}");
        let checksum = fields.get(3).unwrap_or(&"");
        assert_eq!(checksum.len(), 64, "{row}");
        assert!(
            fixture.trim_end().ends_with(checksum),
            "{row}: checksum column drifted"
        );
        rows = rows.saturating_add(1);
    }
    assert_eq!(rows, 3);
    Ok(())
}

#[test]
fn format_registry_routes_to_the_receipt_corpus() -> Result<(), io::Error> {
    let format_index = fs::read_to_string(repository_root()?.join("docs/formats/README.md"))?;
    assert!(
        format_index.contains("../../conformance/verification-receipt/v1/README.md"),
        "format registry does not route to the verification-receipt corpus"
    );
    Ok(())
}
