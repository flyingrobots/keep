//! Independent construction oracle for version-2 segment-store golden bytes.

#![cfg(feature = "repository-tasks")]

const CORPUS_ROOT: &str = "conformance/segment-store/v2";
const PROFILE_DEFINITION: &str =
    include_str!("../../conformance/segment-store/v2/retention-profile.tsv");
const FORMAT_DEFINITION: &str = include_str!("../../conformance/segment-store/v2/definition.tsv");
const LAYOUTS: &str = include_str!("../../conformance/layout/v1/layouts.tsv");

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
    artifacts: Vec<Artifact>,
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
