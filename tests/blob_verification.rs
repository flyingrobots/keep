//! Public laws for evidence established by complete logical verification.

#[path = "retention_closure/memory_stage.rs"]
mod memory_stage;
#[path = "blob_verification/refusal_laws.rs"]
mod refusal_laws;
#[path = "blob_verification/root_laws.rs"]
mod root_laws;
mod support;

use keep::{
    AdmittedLayout, AdmittedSegment, AdmittedSegmentRecord, CanonicalCatalog,
    CanonicalPublicationHead, CatalogGeneration, CatalogSnapshot, ChecksummedPublicationHead,
    LayoutDecodePolicy, LayoutEntryLimit, SegmentReadPolicy, VerificationDepth as Depth,
    VerificationSubject as Subject,
};
use std::error::Error;

// Size: small. Oracle: frozen one-zero layout and bytes; explicit supported
// depth vocabulary. Delete when the complete-blob reporting contract is retired.
#[test]
fn blob_reports_certify_only_the_requested_evidence_in_one_catalog() -> Result<(), Box<dyn Error>> {
    let layout = one_zero()?;
    let canonical = layout.encode_record()?;
    let records = [
        AdmittedSegmentRecord::for_chunk(&[0])?,
        AdmittedSegmentRecord::for_layout(&canonical)?,
    ];
    with_records(&records, |catalog| {
        for depth in [
            Depth::Framing,
            Depth::Checksum,
            Depth::ChunkIdentity,
            Depth::LayoutIdentity,
            Depth::CompleteBlobIdentity,
        ] {
            let report = catalog.verify_blob(layout.target(), depth)?;
            assert_eq!(report.requested(), depth, "original request must survive");
            assert_eq!(
                report.catalog(),
                Some((catalog.generation(), catalog.catalog_digest())),
                "evidence must bind the exact catalog"
            );
            let claims: Vec<_> = report
                .subjects()
                .iter()
                .map(|entry| (entry.subject(), entry.depth()))
                .collect();
            assert_eq!(
                claims,
                [(
                    Subject::Blob {
                        identity: layout.target()
                    },
                    depth
                )],
                "one exact logical subject must be certified at the requested depth"
            );
        }
        Ok(())
    })
}

// Size: small. Oracle: published catalog bindings can exist without closure.
// Delete if layout admission deliberately begins requiring complete chunk closure.
#[test]
fn shallow_blob_evidence_does_not_require_unclaimed_chunk_closure() -> Result<(), Box<dyn Error>> {
    let layout = one_zero()?;
    let canonical = layout.encode_record()?;
    with_records(
        &[AdmittedSegmentRecord::for_layout(&canonical)?],
        |catalog| {
            for depth in [Depth::Framing, Depth::Checksum, Depth::LayoutIdentity] {
                let report = catalog.verify_blob(layout.target(), depth)?;
                assert_eq!(
                    report.subjects().first().ok_or("missing claim")?.depth(),
                    depth,
                    "shallow evidence must not be escalated"
                );
            }
            Ok(())
        },
    )
}

fn one_zero() -> Result<AdmittedLayout, Box<dyn Error>> {
    Ok(AdmittedLayout::decode_record(
        &support::layout_record_bytes("one-zero")?,
        LayoutDecodePolicy::new(LayoutEntryLimit::MAXIMUM),
    )?)
}

fn with_records(
    records: &[AdmittedSegmentRecord<'_>],
    operation: impl FnOnce(&CatalogSnapshot<'_, '_, '_>) -> Result<(), Box<dyn Error>>,
) -> Result<(), Box<dyn Error>> {
    let bytes = memory_stage::segment_bytes(records)?;
    let segments = if records.is_empty() {
        Vec::new()
    } else {
        vec![AdmittedSegment::decode(&bytes, SegmentReadPolicy::MAXIMUM)?]
    };
    let catalog = CanonicalCatalog::from_segments(CatalogGeneration::new(1)?, None, &segments)?;
    let head = CanonicalPublicationHead::for_catalog(catalog.checksummed());
    let snapshot = ChecksummedPublicationHead::decode(head.encoded())?
        .admit(catalog.checksummed().admit(&segments)?)?;
    operation(&snapshot)
}

#[path = "layout_mutations/support.rs"]
mod layout_mutation_support;
#[path = "blob_verification/profile_law.rs"]
mod profile_law;
