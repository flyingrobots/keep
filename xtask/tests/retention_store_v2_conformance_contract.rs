//! Repository-shape evidence for the version-2 segment-store corpus.

#![cfg(feature = "repository-tasks")]

use std::collections::BTreeSet;
use std::ffi::OsString;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const CORPUS_ROOT: &str = "conformance/segment-store/v2";
const REQUIRED_PATHS: &[&str] = &[
    "README.md",
    "ORIGIN.md",
    "definition.tsv",
    "retention-profile.tsv",
    "inventory.tsv",
    "migration-source.tsv",
    "artifacts.tsv",
    "format-marker.hex",
    "migration-intent.hex",
    "migration-receipt.hex",
    "one-anchor-root.hex",
    "one-root-manifest.hex",
    "one-root-head.hex",
];

fn repository_root() -> Result<PathBuf, io::Error> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| io::Error::other("xtask manifest directory has no parent"))
}

#[test]
fn version_two_corpus_has_one_complete_regular_file_shape() -> Result<(), io::Error> {
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
    assert_eq!(observed, expected, "version-2 corpus shape drifted");
    Ok(())
}

#[test]
fn format_registry_routes_to_executable_version_two_evidence() -> Result<(), io::Error> {
    let format_index = fs::read_to_string(repository_root()?.join("docs/formats/README.md"))?;
    assert!(
        format_index.contains("../../conformance/segment-store/v2/README.md"),
        "format registry does not route to the version-2 corpus"
    );
    Ok(())
}
