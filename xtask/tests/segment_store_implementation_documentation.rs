//! Documentation posture laws for the segment-store implementation.

const ROOT_README: &str = include_str!("../../README.md");
const FORMAT_REGISTRY: &str = include_str!("../../docs/formats/README.md");
const FORMAT_README: &str = include_str!("../../docs/formats/segment-store-v1/README.md");
const REQUIREMENTS: &str = include_str!("../../docs/formats/segment-store-v1/requirements.md");
const CORPUS_README: &str = include_str!("../../conformance/segment-store/v1/README.md");

#[test]
fn living_documentation_names_the_implemented_segment_boundary() {
    for (document, claim) in [
        (ROOT_README, "`StagedSegment`"),
        (ROOT_README, "`AdmittedSegment`"),
        (
            FORMAT_REGISTRY,
            "Implemented through initialization, publication, restart, and recovery in issues #14–#17",
        ),
        (
            FORMAT_README,
            "Segment writing and verified reading are implemented in issue #15",
        ),
        (
            CORPUS_README,
            "Production segment codecs and readers consume these bytes",
        ),
        (REQUIREMENTS, "`KEEP-SEGMENT-010`"),
    ] {
        assert!(
            document.contains(claim),
            "missing documentation claim: {claim}"
        );
    }
    assert!(!ROOT_README.contains("Durable segment storage, retention"));
}
