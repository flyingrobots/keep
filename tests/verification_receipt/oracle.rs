//! Handwritten construction of every golden verification receipt from the
//! accepted layout and segment-store corpora, independent of the production
//! encoder.

use std::io;

use crate::support::{decode_hex, domain_hash, invalid_corpus};

const V1_CATALOG_GENERATION_TWO: &str =
    include_str!("../../conformance/segment-store/v1/one-zero-catalog-generation-two.hex");
const V2_ARTIFACTS: &str = include_str!("../../conformance/segment-store/v2/artifacts.tsv");
const CHECKSUM_DOMAIN: &[u8] = b"keep.verification-receipt-checksum/v1\0";

/// The canonical one-zero `BlobId` binary from the accepted layout corpus.
pub(crate) const BLOB_ID: [u8; 59] = [
    0x4b, 0x45, 0x45, 0x50, 0x3a, 0x42, 0x4c, 0x4f, 0x42, 0x3a, 0x49, 0x44, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x01, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x1c, 0xfb, 0x8f, 0xa9, 0xe9,
    0x17, 0xab, 0xa1, 0x5a, 0x1f, 0x59, 0x20, 0x95, 0xf3, 0x77, 0xff, 0x18, 0x07, 0x55, 0xfe, 0x12,
    0x12, 0xb0, 0xd7, 0xd2, 0xec, 0x75, 0x0b, 0xd1, 0x28, 0xb6, 0x06,
];
/// The canonical one-zero `LayoutId` binary from the accepted layout corpus.
pub(crate) const LAYOUT_ID: [u8; 60] = [
    0x4b, 0x45, 0x45, 0x50, 0x3a, 0x4c, 0x41, 0x59, 0x4f, 0x55, 0x54, 0x3a, 0x49, 0x44, 0x00, 0x00,
    0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xdc, 0x88, 0x7d, 0xa2, 0x3f,
    0x1a, 0x74, 0x83, 0x35, 0x9a, 0x78, 0xfc, 0x9a, 0x7f, 0xde, 0x80, 0x03, 0x0e, 0xc2, 0xc4, 0x69,
    0x06, 0x03, 0x80, 0x3f, 0x0a, 0xb7, 0xd0, 0xed, 0xb5, 0x65, 0x75, 0xb8,
];

/// One golden receipt with its fixture name.
pub(crate) struct GoldenReceipt {
    pub(crate) case: &'static str,
    pub(crate) fixture: &'static str,
    pub(crate) bytes: Vec<u8>,
}

/// The scalar fields of one receipt in wire order, before its slots.
struct Scalars {
    outcome: u16,
    depth: u16,
    subject_kind: u16,
    view_kind: u16,
    refusal_class: u16,
    evidence_kind: u16,
    layout_present: u16,
    supported_minimum: u16,
    supported_maximum: u16,
    target_present: u16,
}

/// The durable coordinates the frozen version-2 store publishes: catalog
/// generation two and the generation-one manifest.
pub(crate) struct DurableCoordinates {
    pub(crate) catalog_generation: u64,
    pub(crate) catalog_digest: [u8; 32],
    pub(crate) liveness_generation: u64,
    pub(crate) manifest_digest: [u8; 32],
}

pub(crate) fn durable_coordinates() -> Result<DurableCoordinates, io::Error> {
    let catalog = decode_hex(V1_CATALOG_GENERATION_TWO.trim_end())?;
    let catalog_digest: [u8; 32] = catalog
        .get(320..352)
        .and_then(|slice| slice.try_into().ok())
        .ok_or_else(|| invalid_corpus("generation-two catalog lacks its digest"))?;
    let manifest_row = V2_ARTIFACTS
        .lines()
        .find(|line| line.starts_with("one-root-manifest\t"))
        .ok_or_else(|| invalid_corpus("version-2 artifacts lack the manifest row"))?;
    let manifest_hex = manifest_row
        .split('\t')
        .nth(5)
        .ok_or_else(|| invalid_corpus("manifest row lacks its bound digest"))?;
    let manifest_digest: [u8; 32] = decode_hex(manifest_hex)?
        .try_into()
        .map_err(|_source| invalid_corpus("manifest digest is not 32 bytes"))?;
    Ok(DurableCoordinates {
        catalog_generation: 2,
        catalog_digest,
        liveness_generation: 1,
        manifest_digest,
    })
}

fn assemble(
    scalars: &Scalars,
    subject: &[u8],
    layout: &[u8],
    target: &[u8],
    view: Option<&DurableCoordinates>,
    evidence_index: u64,
    chunks_verified: u64,
) -> Result<Vec<u8>, io::Error> {
    let mut bytes = Vec::with_capacity(384);
    bytes.extend_from_slice(b"KEEP:VERIFY:RCPT");
    bytes.extend_from_slice(&1_u16.to_be_bytes());
    bytes.extend_from_slice(&384_u16.to_be_bytes());
    bytes.extend_from_slice(&0_u32.to_be_bytes());
    bytes.extend_from_slice(&1_u32.to_be_bytes());
    for value in [
        scalars.outcome,
        scalars.depth,
        scalars.subject_kind,
        scalars.view_kind,
        scalars.refusal_class,
        scalars.evidence_kind,
        scalars.layout_present,
        scalars.supported_minimum,
        scalars.supported_maximum,
        scalars.target_present,
    ] {
        bytes.extend_from_slice(&value.to_be_bytes());
    }
    for slot in [subject, layout, target] {
        bytes.extend_from_slice(slot);
        bytes.resize(
            bytes
                .len()
                .saturating_add(60_usize.saturating_sub(slot.len())),
            0,
        );
    }
    match view {
        None => bytes.resize(bytes.len().saturating_add(80), 0),
        Some(view) => {
            bytes.extend_from_slice(&view.catalog_generation.to_be_bytes());
            bytes.extend_from_slice(&view.catalog_digest);
            bytes.extend_from_slice(&view.liveness_generation.to_be_bytes());
            bytes.extend_from_slice(&view.manifest_digest);
        }
    }
    bytes.extend_from_slice(&evidence_index.to_be_bytes());
    bytes.extend_from_slice(&chunks_verified.to_be_bytes());
    bytes.resize(352, 0);
    if bytes.len() != 352 {
        return Err(invalid_corpus("receipt preimage is not 352 bytes"));
    }
    let checksum = domain_hash(CHECKSUM_DOMAIN, &bytes);
    bytes.extend_from_slice(&checksum);
    Ok(bytes)
}

/// Every golden receipt, in `artifacts.tsv` order.
pub(crate) fn golden_receipts() -> Result<Vec<GoldenReceipt>, io::Error> {
    let durable = durable_coordinates()?;
    Ok(vec![
        GoldenReceipt {
            case: "reference-complete-blob-report",
            fixture: "reference-complete-blob-report.hex",
            bytes: assemble(
                &Scalars {
                    outcome: 1,
                    depth: 5,
                    subject_kind: 1,
                    view_kind: 1,
                    refusal_class: 0,
                    evidence_kind: 0,
                    layout_present: 1,
                    supported_minimum: 0,
                    supported_maximum: 0,
                    target_present: 1,
                },
                &BLOB_ID,
                &LAYOUT_ID,
                &BLOB_ID,
                None,
                0,
                1,
            )?,
        },
        GoldenReceipt {
            case: "durable-corrupt-chunk-refusal",
            fixture: "durable-corrupt-chunk-refusal.hex",
            bytes: assemble(
                &Scalars {
                    outcome: 2,
                    depth: 3,
                    subject_kind: 2,
                    view_kind: 2,
                    refusal_class: 2,
                    evidence_kind: 1,
                    layout_present: 1,
                    supported_minimum: 0,
                    supported_maximum: 0,
                    target_present: 0,
                },
                &LAYOUT_ID,
                &LAYOUT_ID,
                &[],
                Some(&durable),
                0,
                0,
            )?,
        },
        GoldenReceipt {
            case: "reference-unsupported-framing-refusal",
            fixture: "reference-unsupported-framing-refusal.hex",
            bytes: assemble(
                &Scalars {
                    outcome: 2,
                    depth: 1,
                    subject_kind: 1,
                    view_kind: 1,
                    refusal_class: 4,
                    evidence_kind: 0,
                    layout_present: 0,
                    supported_minimum: 3,
                    supported_maximum: 5,
                    target_present: 0,
                },
                &BLOB_ID,
                &[],
                &[],
                None,
                0,
                0,
            )?,
        },
    ])
}
