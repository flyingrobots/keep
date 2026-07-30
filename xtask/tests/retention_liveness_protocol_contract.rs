//! Written-contract evidence for retention roots and GC liveness.

#![cfg(feature = "repository-tasks")]

const ADR_INDEX: &str = include_str!("../../docs/adr/README.md");
const ADR: &str = include_str!("../../docs/adr/0009-retention-roots-release-and-gc-liveness.md");

fn normalized(document: &str) -> String {
    document.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[test]
fn retention_liveness_is_one_indexed_cross_cutting_decision() {
    let adr = normalized(ADR);

    for required in [
        "# ADR-0009: Retention Roots, Release, and GC Liveness",
        "- Status: Accepted",
        "[#18](https://github.com/flyingrobots/keep/issues/18)",
        "## Context",
        "## Decision",
        "## Alternatives considered",
        "## Consequences",
    ] {
        assert!(
            adr.contains(required),
            "retention-liveness ADR omits `{required}`"
        );
    }

    assert!(
        ADR_INDEX.contains("0009-retention-roots-release-and-gc-liveness.md"),
        "ADR index omits the retention-liveness decision"
    );
}

#[test]
fn retention_roots_name_verified_reconstruction_anchors() {
    let adr = normalized(ADR);

    for required in [
        "`RetentionNamespace`",
        "`BlobId`",
        "`LayoutId`",
        "reconstruction anchor",
        "canonical closure",
        "`RetentionRealizationProfile`",
        "profile identity, version, and digest",
        "unavailable or mismatched profile",
        "multiple admissible physical representations",
        "nonzero witness count",
        "does not participate in content identity",
        "File existence, catalog membership, and recent access",
        "Echo, Git, Graft, paths, timestamps, or caller identity",
    ] {
        assert!(
            adr.contains(required),
            "retention-root decision omits `{required}`"
        );
    }
}

#[test]
fn generation_transitions_and_gc_snapshots_fail_closed() {
    let adr = normalized(ADR);

    for required in [
        "expected and observed generations",
        "Retention publication holds the same exclusive store writer authority",
        "initial `RootGeneration` is one",
        "`LivenessGeneration`",
        "empty anchor set remains",
        "sorted, duplicate-free",
        "bounded, cycle-safe",
        "fail-closed",
        "implementation-enforced hard ceilings",
        "roots, nodes, depth, encoded bytes, and physical bytes inspected",
        "before traversal or materialization begins",
        "missing or corrupt closure member",
        "immutable liveness snapshot",
        "catalog generation",
        "same exclusive writer authority",
        "from revalidation through physical mutation",
        "Logical retention states apply",
        "Physical GC states apply only to physical material",
        "cannot be inferred from existence, age, recent access",
        "whole immutable segment",
        "any contained record is live",
        "durably published compaction successor",
        "current catalog names no candidate retirement segment",
        "catalog successor is durable before any old segment unlink",
        "`HEAD` never names a catalog with a missing segment",
        "does not promise immediate physical erasure",
    ] {
        assert!(
            adr.contains(required),
            "retention or GC decision omits `{required}`"
        );
    }
}

#[test]
fn alternatives_and_evidence_boundaries_are_explicit() {
    let adr = normalized(ADR);

    for required in [
        "Git refs",
        "Leases",
        "Reference counts",
        "Tracing from explicit roots",
        "physical claim",
        "does not prove application-level meaning",
    ] {
        assert!(
            adr.contains(required),
            "retention-liveness decision omits `{required}`"
        );
    }
}
