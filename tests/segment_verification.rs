//! Public runtime laws for physical segment verification reports.

#[path = "segment_verification/record_laws.rs"]
mod record_laws;
mod support;

use std::error::Error;

use keep::{
    AdmittedSegment, SegmentReadPolicy, VerificationDepth, VerificationRefusal, VerificationSubject,
};
use support::{decode_hex, require_error};

const EMPTY: &str = include_str!("../conformance/segment-store/v1/empty-segment.hex");
const ONE_ZERO: &str = include_str!("../conformance/segment-store/v1/one-zero-segment.hex");

// Size: small. Oracle: frozen physical coordinates in v1/artifacts.tsv;
// only framing/checksum proof is supported for the physical segment subject.
// Delete when this report contract is removed or replaced by a stronger law.
#[test]
fn segment_reports_bind_only_supported_evidence_to_the_exact_physical_bytes()
-> Result<(), Box<dyn Error>> {
    for (fixture, digest) in [
        (
            EMPTY,
            "fef0cccd7bb6f75a3322d5457219ce4875e5f5ae75193e3c66d4856bca380170",
        ),
        (
            ONE_ZERO,
            "b7542dced2ab770894a14d1d04b066e3a899942602c5986d35ba6df6c1a35cfc",
        ),
    ] {
        let bytes = decode_hex(fixture.trim_end())?;
        let segment = AdmittedSegment::decode(&bytes, SegmentReadPolicy::MAXIMUM)?;
        for requested in [VerificationDepth::Framing, VerificationDepth::Checksum] {
            let report = segment.verify(requested)?;
            assert_eq!(report.requested(), requested, "segment request was changed");
            let [verified] = report.subjects() else {
                return Err("segment report must name one subject".into());
            };
            let VerificationSubject::Segment { digest: observed } = verified.subject() else {
                return Err("segment report named a non-segment subject".into());
            };
            assert_eq!(
                observed.as_bytes().as_slice(),
                decode_hex(digest)?,
                "report must name the exact physical segment"
            );
            assert_eq!(
                verified.depth(),
                requested,
                "segment evidence was escalated"
            );
        }
    }
    Ok(())
}

// Size: small. Oracle: a physical segment report is not logical chunk/blob,
// layout, catalog, retention or snapshot evidence, even for an admitted bundle.
// Delete only with an intentional change to this subject-specific contract.
#[test]
fn physical_segment_reports_refuse_logical_or_store_claims() -> Result<(), Box<dyn Error>> {
    let bytes = decode_hex(ONE_ZERO.trim_end())?;
    let segment = AdmittedSegment::decode(&bytes, SegmentReadPolicy::MAXIMUM)?;
    for requested in [
        VerificationDepth::ChunkIdentity,
        VerificationDepth::LayoutIdentity,
        VerificationDepth::CompleteBlobIdentity,
        VerificationDepth::CatalogReachability,
        VerificationDepth::RetentionClosure,
        VerificationDepth::SnapshotBinding,
    ] {
        let refusal = require_error(
            segment.verify(requested),
            "physical evidence certified another subject's claim",
        )?;
        assert_eq!(
            refusal,
            VerificationRefusal::Unsupported {
                subject: VerificationSubject::Segment {
                    digest: segment.digest()
                },
                requested,
                supported: &[VerificationDepth::Framing, VerificationDepth::Checksum],
            },
            "unsupported segment request must retain its exact policy and subject"
        );
    }
    Ok(())
}

// Size: small. Oracle: reporting over admitted immutable evidence is allocation-free.
// Delete only when the documented reporting cost deliberately changes.
#[test]
fn reporting_segment_evidence_allocates_no_heap_memory() -> Result<(), Box<dyn Error>> {
    let bytes = decode_hex(ONE_ZERO.trim_end())?;
    let segment = AdmittedSegment::decode(&bytes, SegmentReadPolicy::MAXIMUM)?;
    let mut result = None;
    let allocation = allocation_counter::measure(|| {
        result = Some(segment.verify(VerificationDepth::Checksum));
    });
    let report = result.ok_or("segment reporting did not execute")??;
    assert_eq!(report.requested(), VerificationDepth::Checksum);
    assert_eq!(
        allocation.bytes_total, 0,
        "segment reporting must not allocate"
    );
    Ok(())
}
