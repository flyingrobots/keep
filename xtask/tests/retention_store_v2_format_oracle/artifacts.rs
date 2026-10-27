// This included source owns handwritten construction of every golden artifact.

const PROFILE_DOMAIN: &[u8] = b"keep.retention-realization-profile/v1\0";
const DEFINITION_DOMAIN: &[u8] = b"keep.segment-store-definition/v2\0";

fn build_corpus() -> Result<Corpus, String> {
    let profile_digest = hash(PROFILE_DOMAIN, &[PROFILE_DEFINITION.as_bytes()]);
    let definition_digest = hash(DEFINITION_DOMAIN, &[FORMAT_DEFINITION.as_bytes()]);
    let format = build_format_marker(definition_digest)?;
    let root = build_retention_root(profile_digest)?;
    let manifest = build_retention_manifest(&root)?;
    let head = build_retention_head(&manifest)?;
    let artifacts = vec![
        format,
        Artifact {
            case_name: "one-anchor-root",
            kind: "retention-root",
            generation: "1",
            entry_count: "1",
            bound_digest: root.digest,
            final_checksum: root.checksum,
            fixture: "one-anchor-root.hex",
            bytes: root.bytes,
        },
        Artifact {
            case_name: "one-root-manifest",
            kind: "retention-manifest",
            generation: "1",
            entry_count: "1",
            bound_digest: manifest.digest,
            final_checksum: manifest.checksum,
            fixture: "one-root-manifest.hex",
            bytes: manifest.bytes,
        },
        head,
    ];
    Ok(Corpus {
        profile_digest,
        definition_digest,
        artifacts,
    })
}

include!("artifacts/format_marker.rs");
include!("artifacts/retention.rs");
