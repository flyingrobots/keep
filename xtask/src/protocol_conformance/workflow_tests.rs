//! Repository-level laws for conformance routing and Python removal.

use std::fs::{self, Metadata};
use std::io;
use std::path::Path;

const CDC_GUIDE: &str = include_str!("../../../conformance/cdc-profile/v1/README.md");
const CHUNK_GUIDE: &str = include_str!("../../../conformance/chunk-id/v1/README.md");
const CI_WORKFLOW: &str = include_str!("../../../.github/workflows/ci.yml");
const COMMAND: &str = "cargo xtask conformance-check";
const CI_RUN_STEP: &str = "run: cargo xtask conformance-check";
const CRASH_MATRIX_DEBUG_STEP: &str = "run: cargo xtask durability-crash-matrix";
const CRASH_MATRIX_RELEASE_STEP: &str =
    "run: cargo run --quiet --release --locked --package xtask -- durability-crash-matrix";

#[test]
fn ci_and_living_guides_route_both_corpora_through_rust() {
    assert!(CI_WORKFLOW.contains("name: Check protocol conformance corpora"));
    assert!(ci_executes_conformance(CI_WORKFLOW));
    assert_eq!(CDC_GUIDE.matches(COMMAND).count(), 1);
    assert_eq!(CHUNK_GUIDE.matches(COMMAND).count(), 1);
    assert!(!CI_WORKFLOW.contains("python3 conformance/"));
    assert!(!CDC_GUIDE.contains("python3"));
    assert!(!CHUNK_GUIDE.contains("python3"));
}

fn ci_executes_conformance(workflow: &str) -> bool {
    workflow
        .lines()
        .filter(|line| line.trim() == CI_RUN_STEP)
        .count()
        == 1
}

#[test]
fn a_commented_command_is_not_ci_execution() {
    assert!(!ci_executes_conformance(
        "# cargo xtask conformance-check\n"
    ));
}

#[test]
fn ci_executes_the_complete_crash_matrix_in_debug_and_optimized_profiles() {
    assert_eq!(
        exact_run_step_count(CI_WORKFLOW, CRASH_MATRIX_DEBUG_STEP),
        1
    );
    assert_eq!(
        exact_run_step_count(CI_WORKFLOW, CRASH_MATRIX_RELEASE_STEP),
        1
    );
}

fn exact_run_step_count(workflow: &str, command: &str) -> usize {
    workflow
        .lines()
        .filter(|line| line.trim() == command)
        .count()
}

#[test]
fn superseded_conformance_python_programs_are_absent() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap_or_else(|| Path::new(""));
    for relative in [
        "conformance/cdc-profile/v1/check_vectors.py",
        "conformance/cdc-profile/v1/scalar_fastcdc.py",
        "conformance/chunk-id/v1/check_vectors.py",
    ] {
        let path = root.join(relative);
        assert!(
            matches!(
                classify_metadata(fs::symlink_metadata(path)),
                PathState::Absent
            ),
            "superseded Python program was not proven absent: {relative}"
        );
    }
}

enum PathState {
    Absent,
    Present,
    Uninspectable(io::Error),
}

fn classify_metadata(result: Result<Metadata, io::Error>) -> PathState {
    match result {
        Ok(_) => PathState::Present,
        Err(source) if source.kind() == io::ErrorKind::NotFound => PathState::Absent,
        Err(source) => PathState::Uninspectable(source),
    }
}

#[test]
fn metadata_failures_are_not_treated_as_absence() {
    let denied = io::Error::from(io::ErrorKind::PermissionDenied);
    assert!(matches!(
        classify_metadata(Err(denied)),
        PathState::Uninspectable(source) if source.kind() == io::ErrorKind::PermissionDenied
    ));
}
