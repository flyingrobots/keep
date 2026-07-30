//! Independent construction oracle for version-2 segment-store golden bytes.

#![cfg(feature = "repository-tasks")]

const CORPUS_ROOT: &str = "conformance/segment-store/v2";
const PROFILE_DEFINITION: &str =
    include_str!("../../conformance/segment-store/v2/retention-profile.tsv");
const FORMAT_DEFINITION: &str = include_str!("../../conformance/segment-store/v2/definition.tsv");
const LAYOUTS: &str = include_str!("../../conformance/layout/v1/layouts.tsv");
const V1_SEGMENT: &str = include_str!("../../conformance/segment-store/v1/one-zero-segment.hex");
const V1_CATALOG: &str = include_str!("../../conformance/segment-store/v1/one-zero-catalog.hex");
const V1_HEAD: &str = include_str!("../../conformance/segment-store/v1/one-zero-head.hex");

struct Artifact {
    case_name: &'static str,
    kind: &'static str,
    generation: &'static str,
    entry_count: &'static str,
    bound_digest: [u8; 32],
    final_checksum: [u8; 32],
    fixture: &'static str,
    bytes: Vec<u8>,
}

struct Corpus {
    profile_digest: [u8; 32],
    definition_digest: [u8; 32],
    inventory: Inventory,
    migration: MigrationSource,
    artifacts: Vec<Artifact>,
}

struct Inventory {
    rows: Vec<InventoryRow>,
    digest: [u8; 32],
}

struct InventoryRow {
    kind: &'static str,
    generation: u64,
    byte_length: u64,
    artifact_digest: [u8; 32],
    source_fixture: &'static str,
    bytes: [u8; 56],
}

struct MigrationSource {
    catalog_generation: u64,
    catalog_length: u64,
    catalog_digest: [u8; 32],
    predecessor_digest: [u8; 32],
    inventory_digest: [u8; 32],
    definition_digest: [u8; 32],
    store_id: [u8; 32],
    root_device: u64,
    root_mount: u64,
    root_file: u64,
}

struct RootArtifact {
    bytes: Vec<u8>,
    digest: [u8; 32],
    checksum: [u8; 32],
    namespace_digest: [u8; 32],
}

struct ManifestArtifact {
    bytes: Vec<u8>,
    digest: [u8; 32],
    checksum: [u8; 32],
}

include!("retention_store_v2_format_oracle/encoding.rs");
include!("retention_store_v2_format_oracle/artifacts.rs");
include!("retention_store_v2_format_oracle/fixture_assertion.rs");
