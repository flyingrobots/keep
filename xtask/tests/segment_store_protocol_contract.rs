//! Written-contract evidence for the durable segment-store protocol.

#![cfg(feature = "repository-tasks")]

use std::path::Path;

#[path = "segment_store_protocol_contract/fixture_oracle.rs"]
mod fixture_oracle;

const ADR_INDEX: &str = include_str!("../../docs/adr/README.md");
const ADR: &str = include_str!("../../docs/adr/0005-durable-segment-store-protocol.md");
const SPECIFICATION_INDEX: &str = include_str!("../../docs/formats/segment-store-v1/README.md");
const SPECIFICATION: &str = concat!(
    include_str!("../../docs/formats/segment-store-v1/README.md"),
    include_str!("../../docs/formats/segment-store-v1/segment.md"),
    include_str!("../../docs/formats/segment-store-v1/catalog.md"),
    include_str!("../../docs/formats/segment-store-v1/publication.md"),
    include_str!("../../docs/formats/segment-store-v1/recovery.md"),
    include_str!("../../docs/formats/segment-store-v1/requirements.md"),
);
const RATIONALE: &str = include_str!("../../docs/formats/segment-store-v1/rationale.md");
const CONFORMANCE_GUIDE: &str = include_str!("../../conformance/segment-store/v1/README.md");
const CONFORMANCE_ORIGIN: &str = include_str!("../../conformance/segment-store/v1/ORIGIN.md");
const TRANSITIONS: &str = include_str!("../../conformance/segment-store/v1/transitions.tsv");

const REQUIRED_PROTOCOL_PAGES: &[&str] = &[
    "docs/adr/0005-durable-segment-store-protocol.md",
    "docs/formats/segment-store-v1/README.md",
    "docs/formats/segment-store-v1/segment.md",
    "docs/formats/segment-store-v1/catalog.md",
    "docs/formats/segment-store-v1/publication.md",
    "docs/formats/segment-store-v1/recovery.md",
    "docs/formats/segment-store-v1/requirements.md",
    "docs/formats/segment-store-v1/rationale.md",
    "conformance/segment-store/v1/README.md",
    "conformance/segment-store/v1/ORIGIN.md",
];

#[test]
fn durable_protocol_is_one_cross_cutting_decision_with_owned_evidence() -> Result<(), &'static str>
{
    let repository_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("xtask manifest must have a repository parent")?;

    for relative_path in REQUIRED_PROTOCOL_PAGES {
        assert!(
            repository_root.join(relative_path).is_file(),
            "missing durable protocol artifact: {relative_path}"
        );
    }

    Ok(())
}

#[test]
fn durable_protocol_freezes_every_cross_cutting_law() {
    for required in [
        "KEEP:SEGMENT:V1",
        "KEEP:SEG:RECORD",
        "KEEP:SEGMENT:END",
        "KEEP:CATALOG:V1",
        "KEEP:CATHEAD:V1",
        "KEEP:SEG:RECORD:SUM\\0",
        "KEEP:SEGMENT:DIGEST\\0",
        "KEEP:SEGMENT:SEAL:SUM\\0",
        "KEEP:CATALOG:SUM\\0",
        "KEEP:CATALOG:DIGEST\\0",
        "KEEP:CATHEAD:SUM\\0",
        "one-writer",
        "atomically hard-link",
        "never overwrites an immutable-pool name",
        "closes every writable staging handle",
        "Every write handles short writes",
        "Opening is read-only",
        "Unrecoverable ambiguity",
    ] {
        assert!(
            SPECIFICATION.contains(required),
            "missing normative durable-store law: {required}"
        );
    }

    assert!(ADR_INDEX.contains("0005-durable-segment-store-protocol.md"));
    assert!(ADR.contains("## Alternatives considered"));
    assert!(RATIONALE.contains("## Observation before recovery"));
    assert!(CONFORMANCE_GUIDE.contains("implementation-independent corpus"));
}

#[test]
fn durable_transition_ledger_is_complete_and_stable() -> Result<(), String> {
    assert!(TRANSITIONS.starts_with(
        "keep.segment-store.transitions/v1\n\
         crash_id\tphase\toperation\tpre_state\tinterrupted_class\t\
         post_state\trecovery_posture\n"
    ));

    let mut row_count = 0usize;
    for (offset, row) in TRANSITIONS.lines().skip(2).enumerate() {
        let ordinal = offset.checked_add(1).ok_or("transition ordinal overflow")?;
        let expected_id = format!("KEEP-CRASH-{ordinal:03}");
        let mut fields = row.split('\t');
        assert_eq!(fields.next(), Some(expected_id.as_str()));
        assert_eq!(
            fields.count(),
            6,
            "transition {expected_id} must have seven fields"
        );
        row_count = row_count
            .checked_add(1)
            .ok_or("transition count overflow")?;
    }
    assert_eq!(row_count, 28);
    for exact_transition in [
        "KEEP-CRASH-009\tsegment\tlink-sealed-stage\t\
         durable-sealed-stage\t\
         valid-sealed-stage-or-valid-orphan-or-ambiguity",
        "KEEP-CRASH-017\tcatalog\tlink-generation\t\
         durable-catalog-stage\t\
         valid-catalog-stage-or-valid-orphan-or-ambiguity",
        "KEEP-CRASH-025\thead\treplace-current-head\t\
         durable-next-head\t\
         valid-next-head-or-published-generation-or-ambiguity",
        "KEEP-CRASH-026\thead\tsync-root-directory\t\
         replaced-current-head\tpublished-generation-or-ambiguity",
        "KEEP-CRASH-027\trecovery\tunlink-truncated-stage\t\
         named-truncated-stage\t\
         named-truncated-stage-or-unlinked-stage",
        "KEEP-CRASH-028\trecovery\tsync-staging-after-discard\t\
         unlinked-truncated-stage\t\
         named-truncated-stage-or-discarded-stage",
    ] {
        assert!(
            TRANSITIONS.contains(exact_transition),
            "imprecise atomic-transition state: {exact_transition}"
        );
    }

    for recovery_class in [
        "reusable-stage",
        "valid-orphan",
        "truncated-tail",
        "corrupt",
        "stale-generation",
        "ambiguity",
    ] {
        assert!(
            format!("{SPECIFICATION}\n{TRANSITIONS}").contains(recovery_class),
            "missing recovery class: {recovery_class}"
        );
    }

    Ok(())
}

#[test]
fn recovery_inventory_is_bounded_before_names_are_retained() {
    for required in [
        "`MAX_RECOVERY_INVENTORY_ENTRY_COUNT` | `2,097,152`",
        "recovery counts entries",
        "before retaining or sorting their names",
        "observed-at-least",
        "`2,097,153`",
    ] {
        assert!(
            SPECIFICATION.contains(required),
            "missing bounded recovery-inventory law: {required}"
        );
    }
}

#[test]
fn catalog_locations_name_only_top_level_segment_records() {
    for required in [
        "scans the complete segment grammar from byte 64",
        "must equal one discovered",
        "top-level record span",
        "record header, payload, checksum",
        "segment seal is refused",
        "`KEEP-STORE-017`",
    ] {
        assert!(
            SPECIFICATION.contains(required),
            "missing top-level catalog-span law: {required}"
        );
    }
}

#[test]
fn conformance_provenance_has_one_issue_prefix_per_owner() {
    let normalized = CONFORMANCE_ORIGIN
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    assert!(
        !normalized.contains("Issue Issue"),
        "conformance provenance repeats an issue prefix"
    );
}

#[test]
fn physical_namespace_refuses_aliasing_filesystems() {
    for required in [
        "case-sensitive, byte-preserving directory names",
        "case-folding or normalization aliases",
    ] {
        assert!(
            SPECIFICATION.contains(required),
            "missing physical-namespace capability: {required}"
        );
    }
}

#[test]
fn protocol_index_routes_each_semantic_owner() {
    assert!(
        SPECIFICATION_INDEX.lines().count() <= 200,
        "protocol index exceeds the repository target file size"
    );
    for page in [
        "segment.md",
        "catalog.md",
        "publication.md",
        "recovery.md",
        "requirements.md",
    ] {
        assert!(
            SPECIFICATION_INDEX.contains(page),
            "protocol index does not route to {page}"
        );
    }
}

#[test]
fn truncated_stage_discard_fingerprint_has_one_preimage() {
    assert!(
        SPECIFICATION.contains("framed_blake3_v1(ASCII(\"KEEP:RECOVERY:STAGE\\0\"), stage_bytes)"),
        "truncated-stage discard fingerprint lacks its exact preimage"
    );
}
