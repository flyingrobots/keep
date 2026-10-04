//! Public laws preventing logical-record evidence from claiming closure.

use std::error::Error;

use crate::support::{decode_hex, layout_case_field, layout_record_bytes, require_error};
use keep::{
    AdmittedLayout, AdmittedSegment, AdmittedSegmentRecord, ChunkId, LayoutDecodePolicy,
    LayoutEntryLimit, SegmentReadPolicy, VerificationDepth, VerificationRefusal,
    VerificationReport, VerificationSubject,
};

const BUNDLE: &str = include_str!("../../conformance/segment-store/v1/one-zero-bundle-segment.hex");
const DEPTHS: [VerificationDepth; 8] = [
    VerificationDepth::Framing,
    VerificationDepth::Checksum,
    VerificationDepth::ChunkIdentity,
    VerificationDepth::LayoutIdentity,
    VerificationDepth::CompleteBlobIdentity,
    VerificationDepth::CatalogReachability,
    VerificationDepth::RetentionClosure,
    VerificationDepth::SnapshotBinding,
];

// Size: small. Oracle: the one-zero bundle contains a chunk and its canonical
// layout; each supports only its specified framing/checksum/logical proof.
// Delete when the record report contract is deliberately replaced.
#[test]
fn admitted_record_reports_preserve_each_logical_subjects_exact_proof_scope()
-> Result<(), Box<dyn Error>> {
    let bytes = decode_hex(BUNDLE.trim_end())?;
    let segment = AdmittedSegment::decode(&bytes, SegmentReadPolicy::MAXIMUM)?;
    let mut records = segment.records();
    let chunk = records
        .next()
        .ok_or("frozen bundle lost its chunk witness")??;
    let layout = records
        .next()
        .ok_or("frozen bundle lost its layout witness")??;
    let expected_chunk = VerificationSubject::Chunk {
        identity: ChunkId::hash_bytes(&[0])?,
    };
    let expected_layout = VerificationSubject::Layout {
        identity: layout_case_field("one-zero", 10)?.parse()?,
    };
    assert_record_depths(
        chunk,
        expected_chunk,
        &[
            VerificationDepth::Framing,
            VerificationDepth::Checksum,
            VerificationDepth::ChunkIdentity,
        ],
    )?;
    assert_record_depths(
        layout,
        expected_layout,
        &[
            VerificationDepth::Framing,
            VerificationDepth::Checksum,
            VerificationDepth::LayoutIdentity,
        ],
    )?;
    Ok(())
}

fn assert_record_depths(
    record: AdmittedSegmentRecord<'_>,
    subject: VerificationSubject,
    supported: &'static [VerificationDepth],
) -> Result<(), Box<dyn Error>> {
    for requested in DEPTHS {
        if supported.contains(&requested) {
            assert_report(&record.verify(requested)?, subject, requested)?;
        } else {
            let refusal = require_error(
                record.verify(requested),
                "record evidence certified an unsupported claim",
            )?;
            assert_eq!(
                refusal,
                VerificationRefusal::Unsupported {
                    subject,
                    requested,
                    supported
                },
                "record refusal must preserve subject, requested depth and exact supported set"
            );
        }
    }
    Ok(())
}

fn assert_report(
    report: &VerificationReport,
    expected: VerificationSubject,
    requested: VerificationDepth,
) -> Result<(), Box<dyn Error>> {
    assert_eq!(report.requested(), requested, "record request was changed");
    let [verified] = report.subjects() else {
        return Err("record report must name one subject".into());
    };
    assert_eq!(
        (verified.subject(), verified.depth()),
        (expected, requested),
        "record report must name its exact subject and established proof"
    );
    Ok(())
}

// Size: small. Oracle: preparing a layout does not load its referenced chunks;
// layout identity is valid without complete-blob evidence or publication.
// Delete only if layout preparation explicitly acquires and verifies closure.
#[test]
fn a_layout_prepared_without_its_chunks_cannot_report_complete_blob_verification()
-> Result<(), Box<dyn Error>> {
    let admitted = AdmittedLayout::decode_record(
        &layout_record_bytes("one-zero")?,
        LayoutDecodePolicy::new(LayoutEntryLimit::MAXIMUM),
    )?;
    let canonical = admitted.encode_record()?;
    let record = AdmittedSegmentRecord::for_layout(&canonical)?;
    let subject = VerificationSubject::Layout {
        identity: layout_case_field("one-zero", 10)?.parse()?,
    };
    assert_report(
        &record.verify(VerificationDepth::LayoutIdentity)?,
        subject,
        VerificationDepth::LayoutIdentity,
    )?;
    let refusal = require_error(
        record.verify(VerificationDepth::CompleteBlobIdentity),
        "layout identity was presented as complete-blob evidence",
    )?;
    assert_eq!(
        refusal,
        VerificationRefusal::Unsupported {
            subject,
            requested: VerificationDepth::CompleteBlobIdentity,
            supported: &[
                VerificationDepth::Framing,
                VerificationDepth::Checksum,
                VerificationDepth::LayoutIdentity
            ],
        }
    );
    Ok(())
}

// Size: small. Oracle: reporting over admitted logical evidence allocates nothing.
// Delete only when the public reporting cost intentionally changes.
#[test]
fn reporting_logical_record_evidence_allocates_no_heap_memory() -> Result<(), Box<dyn Error>> {
    let record = AdmittedSegmentRecord::for_chunk(&[0])?;
    let mut result = None;
    let allocation = allocation_counter::measure(|| {
        result = Some(record.verify(VerificationDepth::ChunkIdentity));
    });
    let report = result.ok_or("record reporting did not execute")??;
    assert_eq!(report.requested(), VerificationDepth::ChunkIdentity);
    assert_eq!(
        allocation.bytes_total, 0,
        "record reporting must not allocate"
    );
    Ok(())
}
