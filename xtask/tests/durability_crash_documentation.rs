//! Documentation truth laws for the process-death crash matrix.

#![cfg(feature = "repository-tasks")]

const ROOT_README: &str = include_str!("../../README.md");
const RECOVERY: &str = include_str!("../../docs/formats/segment-store-v1/recovery.md");
const REQUIREMENTS: &str = include_str!("../../docs/formats/segment-store-v1/requirements.md");
const CORPUS_README: &str = include_str!("../../conformance/segment-store/v1/README.md");
const V2_CORPUS_README: &str = include_str!("../../conformance/segment-store/v2/README.md");
const INITIALIZATION: &str = include_str!("../../docs/formats/segment-store-v2/initialization.md");
const V2_REQUIREMENTS: &str = include_str!("../../docs/formats/segment-store-v2/requirements.md");

#[test]
fn living_documentation_routes_the_complete_crash_matrix_and_its_limits() {
    for (document, claim) in [
        (ROOT_README, "cargo xtask durability-crash-matrix"),
        (RECOVERY, "## Process-death crash matrix"),
        (REQUIREMENTS, "`KEEP-RECOVERY-021`"),
        (CORPUS_README, "105 canonical process-death cases"),
        (
            V2_CORPUS_README,
            "41 canonical initialization process-death cases",
        ),
        (
            INITIALIZATION,
            "cargo xtask durability-crash-matrix --sequence version-two-initialization",
        ),
        (V2_REQUIREMENTS, "`KEEP-INITIALIZATION-004`"),
    ] {
        assert!(
            document.contains(claim),
            "missing crash-matrix documentation claim: {claim}"
        );
    }
    assert!(!ROOT_README.contains(
        "Process-death injection, retention, compaction, and garbage collection remain planned."
    ));
    assert!(RECOVERY.contains("does not simulate host power loss"));
    assert!(INITIALIZATION.contains("It is process-death evidence."));
}
