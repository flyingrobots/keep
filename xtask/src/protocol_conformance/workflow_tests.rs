//! Repository-level laws for conformance routing and Python removal.

use std::path::Path;

const CDC_GUIDE: &str = include_str!("../../../conformance/cdc-profile/v1/README.md");
const CHUNK_GUIDE: &str = include_str!("../../../conformance/chunk-id/v1/README.md");
const CI_WORKFLOW: &str = include_str!("../../../.github/workflows/ci.yml");
const COMMAND: &str = "cargo xtask conformance-check";

#[test]
fn ci_and_living_guides_route_both_corpora_through_rust() {
    assert!(CI_WORKFLOW.contains("name: Check protocol conformance corpora"));
    assert_eq!(CI_WORKFLOW.matches(COMMAND).count(), 1);
    assert_eq!(CDC_GUIDE.matches(COMMAND).count(), 1);
    assert_eq!(CHUNK_GUIDE.matches(COMMAND).count(), 1);
    assert!(!CI_WORKFLOW.contains("python3 conformance/"));
    assert!(!CDC_GUIDE.contains("python3"));
    assert!(!CHUNK_GUIDE.contains("python3"));
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
        assert!(
            !root.join(relative).exists(),
            "superseded Python program remains: {relative}"
        );
    }
}
