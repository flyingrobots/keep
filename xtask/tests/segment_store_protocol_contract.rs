//! Written-contract evidence for the durable segment-store protocol.

#![cfg(feature = "repository-tasks")]

use std::path::Path;

#[path = "segment_store_protocol_contract/documentation_laws.rs"]
mod documentation_laws;
#[path = "segment_store_protocol_contract/fixture_oracle.rs"]
mod fixture_oracle;
#[path = "segment_store_protocol_contract/publication_laws.rs"]
mod publication_laws;
#[path = "segment_store_protocol_contract/recovery_laws.rs"]
mod recovery_laws;
#[path = "segment_store_protocol_contract/transition_laws.rs"]
mod transition_laws;

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
const CONTRACT_SOURCE: &str = include_str!("segment_store_protocol_contract.rs");

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
fn catalog_locations_name_only_top_level_segment_records() {
    for required in [
        "scans each referenced\nsegment exactly once",
        "Each scan runs from byte 64",
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
