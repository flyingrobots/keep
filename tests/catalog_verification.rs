//! Public laws for catalog-specific verification reports.

#[path = "retention_closure/memory_stage.rs"]
mod memory_stage;
mod support;

use std::error::Error;

use keep::{
    AdmittedLayout, AdmittedSegment, AdmittedSegmentRecord, CanonicalCatalog,
    CanonicalPublicationHead, CatalogGeneration, CatalogSnapshot, ChecksummedCatalog,
    ChecksummedPublicationHead, LayoutDecodePolicy, LayoutEntryLimit, SegmentReadPolicy,
    SegmentRecordIdentity, VerificationDepth, VerificationRefusal, VerificationSubject,
};
use support::{decode_hex, layout_record_bytes, require_error};

const CATALOG: &str =
    include_str!("../conformance/segment-store/v1/one-zero-catalog-generation-two.hex");
const HEAD: &str = include_str!("../conformance/segment-store/v1/one-zero-head-generation-two.hex");
const SEGMENT: &str = include_str!("../conformance/segment-store/v1/one-zero-segment.hex");
const CATALOG_DIGEST: &str = "ea7d0055fd21f00ed94809ef4e671d72fa2e6a4a5d9ecefb23f3a320a2dad993";

// Size: small. Oracle: frozen generation-two corpus coordinates and the public
// contract that successful reports state the requested catalog evidence only.
// Delete when catalog verification is removed or a stronger report law subsumes it.
#[test]
fn catalog_reports_bind_the_requested_evidence_to_the_selected_generation()
-> Result<(), Box<dyn Error>> {
    with_snapshot(|snapshot| {
        for depth in [
            VerificationDepth::Framing,
            VerificationDepth::Checksum,
            VerificationDepth::CatalogReachability,
        ] {
            let report = snapshot.verify(depth)?;
            assert_eq!(
                report.requested(),
                depth,
                "original request must be retained"
            );
            let observed: Vec<_> = report
                .subjects()
                .iter()
                .map(|verified| (verified.subject(), verified.depth()))
                .collect();
            assert_eq!(
                observed,
                [(expected_subject(snapshot)?, depth)],
                "report must contain exactly the selected catalog's requested evidence"
            );
        }
        Ok(())
    })
}

// Size: small. Oracle: catalog admission does not establish logical blob or
// retention/snapshot closure; unsupported requests must preserve exact coordinates.
// Delete when these subject/depth contracts are deliberately replaced.
#[test]
fn catalog_requests_outside_its_evidence_refuse_without_downgrading() -> Result<(), Box<dyn Error>>
{
    with_snapshot(|snapshot| {
        for requested in [
            VerificationDepth::ChunkIdentity,
            VerificationDepth::LayoutIdentity,
            VerificationDepth::CompleteBlobIdentity,
            VerificationDepth::RetentionClosure,
            VerificationDepth::SnapshotBinding,
        ] {
            let refusal = require_error(snapshot.verify(requested), "unsupported depth certified")?;
            assert_eq!(
                refusal,
                VerificationRefusal::Unsupported {
                    subject: expected_subject(snapshot)?,
                    requested,
                    supported: &[
                        VerificationDepth::Framing,
                        VerificationDepth::Checksum,
                        VerificationDepth::CatalogReachability,
                    ],
                },
                "unsupported request must name its subject, request, and exact supported set"
            );
        }
        Ok(())
    })
}

// Size: small. Oracle: catalog binding admits layout records independently of
// chunk closure. The missing chunk is deliberate, not a fabricated report.
// Delete when catalog admission requires complete blob closure by contract.
#[test]
fn catalog_reachability_does_not_certify_an_incomplete_blob() -> Result<(), Box<dyn Error>> {
    let layout = AdmittedLayout::decode_record(
        &layout_record_bytes("one-zero")?,
        LayoutDecodePolicy::new(LayoutEntryLimit::MAXIMUM),
    )?;
    let chunk = layout
        .entries()
        .first()
        .ok_or("missing fixture chunk")?
        .chunk_id();
    let canonical = layout.encode_record()?;
    let bytes = memory_stage::segment_bytes(&[AdmittedSegmentRecord::for_layout(&canonical)?])?;
    let segments = [AdmittedSegment::decode(&bytes, SegmentReadPolicy::MAXIMUM)?];
    let catalog = CanonicalCatalog::from_segments(CatalogGeneration::new(1)?, None, &segments)?;
    let head = CanonicalPublicationHead::for_catalog(catalog.checksummed());
    let snapshot = ChecksummedPublicationHead::decode(head.encoded())?
        .admit(catalog.checksummed().admit(&segments)?)?;

    assert!(
        snapshot
            .record(SegmentRecordIdentity::Chunk(chunk))
            .is_none()
    );
    let report = snapshot.verify(VerificationDepth::CatalogReachability)?;
    assert_eq!(report.requested(), VerificationDepth::CatalogReachability);
    let refusal = require_error(
        snapshot.verify(VerificationDepth::CompleteBlobIdentity),
        "catalog membership was presented as complete blob verification",
    )?;
    assert!(
        matches!(
            refusal,
            VerificationRefusal::Unsupported {
                requested: VerificationDepth::CompleteBlobIdentity,
                ..
            }
        ),
        "an incomplete blob must not receive a complete-identity report: {refusal:?}"
    );
    Ok(())
}

// Size: small. Oracle: producing a report over admitted evidence allocates nothing.
// Delete if the public reporting contract explicitly permits allocation.
#[test]
fn reporting_catalog_evidence_requires_no_additional_allocation() -> Result<(), Box<dyn Error>> {
    with_snapshot(|snapshot| {
        let mut result = None;
        let allocation = allocation_counter::measure(|| {
            result = Some(snapshot.verify(VerificationDepth::CatalogReachability));
        });
        let report = result.ok_or("reporting did not execute")??;
        assert_eq!(report.requested(), VerificationDepth::CatalogReachability);
        assert_eq!(allocation.bytes_total, 0, "reporting must not allocate");
        Ok(())
    })
}

fn with_snapshot(
    operation: impl FnOnce(&CatalogSnapshot<'_, '_, '_>) -> Result<(), Box<dyn Error>>,
) -> Result<(), Box<dyn Error>> {
    let catalog = fixture(CATALOG)?;
    let head = fixture(HEAD)?;
    let segment = fixture(SEGMENT)?;
    let segments = [AdmittedSegment::decode(
        &segment,
        SegmentReadPolicy::MAXIMUM,
    )?];
    let snapshot = ChecksummedPublicationHead::decode(&head)?
        .admit(ChecksummedCatalog::decode(&catalog)?.admit(&segments)?)?;
    operation(&snapshot)
}

fn expected_subject(
    snapshot: &CatalogSnapshot<'_, '_, '_>,
) -> Result<VerificationSubject, Box<dyn Error>> {
    assert_eq!(
        snapshot.catalog_digest().as_bytes().as_slice(),
        decode_hex(CATALOG_DIGEST)?
    );
    Ok(VerificationSubject::Catalog {
        generation: CatalogGeneration::new(2)?,
        digest: snapshot.catalog_digest(),
    })
}

fn fixture(hex: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    decode_hex(hex.strip_suffix('\n').ok_or("fixture lacks terminal LF")?).map_err(Into::into)
}
