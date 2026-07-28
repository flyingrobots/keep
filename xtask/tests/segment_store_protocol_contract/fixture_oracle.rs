//! Test-only construction oracle for durable segment-store golden bytes.

const ARTIFACTS: &str = include_str!("../../../conformance/segment-store/v1/artifacts.tsv");
const EMPTY_SEGMENT: &str = include_str!("../../../conformance/segment-store/v1/empty-segment.hex");
const ONE_ZERO_SEGMENT: &str =
    include_str!("../../../conformance/segment-store/v1/one-zero-segment.hex");
const ONE_ZERO_CATALOG: &str =
    include_str!("../../../conformance/segment-store/v1/one-zero-catalog.hex");
const ONE_ZERO_HEAD: &str = include_str!("../../../conformance/segment-store/v1/one-zero-head.hex");
const ONE_ZERO_BUNDLE_SEGMENT: &str =
    include_str!("../../../conformance/segment-store/v1/one-zero-bundle-segment.hex");
const ONE_ZERO_BUNDLE_CATALOG: &str =
    include_str!("../../../conformance/segment-store/v1/one-zero-bundle-catalog.hex");
const ONE_ZERO_BUNDLE_HEAD: &str =
    include_str!("../../../conformance/segment-store/v1/one-zero-bundle-head.hex");
struct Artifact {
    case_name: &'static str,
    kind: &'static str,
    record_count: &'static str,
    entry_count: &'static str,
    generation: &'static str,
    bound_digest: [u8; 32],
    final_checksum: [u8; 32],
    fixture: &'static str,
    bytes: Vec<u8>,
}

include!("fixture_oracle/encoding.rs");
include!("fixture_oracle/bundle_encoding.rs");
include!("fixture_oracle/fixture_assertion.rs");

fn expected_artifacts() -> Result<Vec<Artifact>, &'static str> {
    let mut artifacts = vec![empty_segment_artifact()?];
    artifacts.extend(one_zero_artifacts()?);
    artifacts.extend(one_zero_bundle_artifacts()?);
    Ok(artifacts)
}

fn empty_segment_artifact() -> Result<Artifact, &'static str> {
    let empty_segment = build_empty_segment()?;
    Ok(Artifact {
        case_name: "empty-segment",
        kind: "segment",
        record_count: "0",
        entry_count: "-",
        generation: "-",
        bound_digest: empty_segment.digest,
        final_checksum: empty_segment.seal_checksum,
        fixture: "empty-segment.hex",
        bytes: empty_segment.bytes,
    })
}

fn one_zero_artifacts() -> Result<[Artifact; 3], &'static str> {
    let one_zero_segment = build_one_zero_segment()?;
    let record_checksum = one_zero_segment
        .record_checksum
        .ok_or("one-zero segment must have a record checksum")?;
    let catalog = build_catalog(one_zero_segment.digest, record_checksum)?;
    let head = build_head_for_catalog(&catalog)?;

    Ok([
        Artifact {
            case_name: "one-zero-segment",
            kind: "segment",
            record_count: "1",
            entry_count: "-",
            generation: "-",
            bound_digest: one_zero_segment.digest,
            final_checksum: one_zero_segment.seal_checksum,
            fixture: "one-zero-segment.hex",
            bytes: one_zero_segment.bytes,
        },
        Artifact {
            case_name: "one-zero-catalog",
            kind: "catalog",
            record_count: "-",
            entry_count: "1",
            generation: "1",
            bound_digest: catalog.digest,
            final_checksum: catalog.checksum,
            fixture: "one-zero-catalog.hex",
            bytes: catalog.bytes,
        },
        Artifact {
            case_name: "one-zero-head",
            kind: "head",
            record_count: "-",
            entry_count: "-",
            generation: "1",
            bound_digest: catalog.digest,
            final_checksum: head.1,
            fixture: "one-zero-head.hex",
            bytes: head.0,
        },
    ])
}

fn one_zero_bundle_artifacts() -> Result<[Artifact; 3], &'static str> {
    let bundle = build_one_zero_bundle_segment()?;
    let bundle_catalog = build_bundle_catalog(
        bundle.segment.digest,
        bundle.chunk_checksum,
        bundle.layout_checksum,
    )?;
    let bundle_head = build_head_for_catalog(&bundle_catalog)?;

    Ok([
        Artifact {
            case_name: "one-zero-bundle-segment",
            kind: "segment",
            record_count: "2",
            entry_count: "-",
            generation: "-",
            bound_digest: bundle.segment.digest,
            final_checksum: bundle.segment.seal_checksum,
            fixture: "one-zero-bundle-segment.hex",
            bytes: bundle.segment.bytes,
        },
        Artifact {
            case_name: "one-zero-bundle-catalog",
            kind: "catalog",
            record_count: "-",
            entry_count: "2",
            generation: "1",
            bound_digest: bundle_catalog.digest,
            final_checksum: bundle_catalog.checksum,
            fixture: "one-zero-bundle-catalog.hex",
            bytes: bundle_catalog.bytes,
        },
        Artifact {
            case_name: "one-zero-bundle-head",
            kind: "head",
            record_count: "-",
            entry_count: "-",
            generation: "1",
            bound_digest: bundle_catalog.digest,
            final_checksum: bundle_head.1,
            fixture: "one-zero-bundle-head.hex",
            bytes: bundle_head.0,
        },
    ])
}
