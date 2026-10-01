//! Documentation posture laws for the segment-store implementation.

const ROOT_README: &str = include_str!("../../README.md");
const FORMAT_REGISTRY: &str = include_str!("../../docs/formats/README.md");
const FORMAT_README: &str = include_str!("../../docs/formats/segment-store-v1/README.md");
const REQUIREMENTS: &str = include_str!("../../docs/formats/segment-store-v1/requirements.md");
const PUBLICATION: &str = include_str!("../../docs/formats/segment-store-v1/publication.md");
const RECOVERY: &str = include_str!("../../docs/formats/segment-store-v1/recovery.md");
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

#[test]
fn living_v1_pages_no_longer_assign_shipped_recovery_to_a_future_issue() {
    for (document, stale_claim) in [
        (FORMAT_README, "remain owned by issue #17"),
        (PUBLICATION, "Issue #17 must implement initialization"),
        (
            PUBLICATION,
            "Issue #16 does not implement store-root initialization",
        ),
        (PUBLICATION, "A future admission producer"),
        (RECOVERY, "remain unimplemented"),
        (REQUIREMENTS, "Explicit\nrecovery remains separate work"),
        (REQUIREMENTS, "remain\nowned by issue #17"),
    ] {
        assert!(
            !document.contains(stale_claim),
            "stale issue-era claim survives: {stale_claim:?}"
        );
    }
}

#[test]
fn living_v2_pages_and_crate_docs_agree_with_the_shipped_recovery_boundary() {
    for (document, stale_claim) in [
        (
            include_str!("../../src/lib.rs"),
            "execution remain intentionally absent",
        ),
        (
            include_str!("../../docs/formats/segment-store-v2/README.md"),
            "planned in issue #19",
        ),
        (
            include_str!("../../docs/formats/segment-store-v2/retention.md"),
            "collection remain absent",
        ),
        (
            include_str!("../../docs/formats/segment-store-v2/closure-corruption.md"),
            "future\nretention publication adapter",
        ),
        (
            include_str!("../../docs/formats/segment-store-v2/retention-publication.md"),
            "No version-2 catalog publisher exists yet",
        ),
    ] {
        assert!(
            !document.contains(stale_claim),
            "shipped behavior described as missing: {stale_claim}"
        );
    }
}
