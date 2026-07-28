//! Protocol navigation and provenance laws.

use super::{CONFORMANCE_ORIGIN, CONTRACT_SOURCE, SPECIFICATION_INDEX};

const FIXTURE_ENCODING: &str = include_str!("fixture_oracle/encoding.rs");
const FIXTURE_CATALOG_ENCODING: &str = include_str!("fixture_oracle/catalog_encoding.rs");
const FIXTURE_BUNDLE_ENCODING: &str = include_str!("fixture_oracle/bundle_encoding.rs");
const FIXTURE_ASSERTION: &str = include_str!("fixture_oracle/fixture_assertion.rs");

#[test]
fn fixture_oracle_modules_stay_within_the_source_target() {
    for (name, source) in [
        ("encoding", FIXTURE_ENCODING),
        ("catalog_encoding", FIXTURE_CATALOG_ENCODING),
        ("bundle_encoding", FIXTURE_BUNDLE_ENCODING),
        ("fixture_assertion", FIXTURE_ASSERTION),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count <= 200,
            "{name} has {line_count} lines; target is 200"
        );
    }
}

#[test]
fn conformance_provenance_has_one_issue_prefix_per_owner() {
    let normalized = CONFORMANCE_ORIGIN
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    for owner in ["Issue #15", "Issue #16", "Issue #17"] {
        assert_eq!(
            normalized.matches(owner).count(),
            1,
            "conformance provenance must name {owner} exactly once"
        );
    }
    assert_eq!(
        normalized.matches("flyingrobots/keep/issues/14").count(),
        1,
        "conformance provenance must name issue #14 exactly once"
    );
}

#[test]
fn protocol_index_routes_each_semantic_owner() {
    assert!(
        SPECIFICATION_INDEX.lines().count() <= 200,
        "protocol index exceeds the repository target file size"
    );
    for route in [
        "[Segment bytes](segment.md)",
        "[Catalog and publication-head bytes](catalog.md)",
        "[Publication and reader visibility](publication.md)",
        "[Recovery and platform contract](recovery.md)",
        "[Requirements, compatibility, and evidence](requirements.md)",
    ] {
        assert!(
            SPECIFICATION_INDEX.contains(route),
            "protocol index does not preserve the exact route: {route}"
        );
    }
}

#[test]
fn contract_root_stays_below_the_review_threshold() {
    assert!(
        CONTRACT_SOURCE.lines().count() <= 300,
        "segment-store protocol contract root requires semantic decomposition"
    );
}
