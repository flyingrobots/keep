//! Documentation truth laws for the process-death crash matrix.

const ROOT_README: &str = include_str!("../../README.md");
const RECOVERY: &str = include_str!("../../docs/formats/segment-store-v1/recovery.md");
const REQUIREMENTS: &str = include_str!("../../docs/formats/segment-store-v1/requirements.md");
const CORPUS_README: &str = include_str!("../../conformance/segment-store/v1/README.md");

#[test]
fn living_documentation_routes_the_complete_crash_matrix_and_its_limits() {
    for (document, claim) in [
        (ROOT_README, "cargo xtask durability-crash-matrix"),
        (RECOVERY, "## Process-death crash matrix"),
        (REQUIREMENTS, "`KEEP-RECOVERY-021`"),
        (CORPUS_README, "105 canonical process-death cases"),
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
}
