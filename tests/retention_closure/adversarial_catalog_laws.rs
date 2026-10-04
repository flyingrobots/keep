//! Adversarial catalog and first-refusal ordering laws.

use std::error::Error;

use keep::{
    AdmittedLayout, AdmittedSegment, AdmittedSegmentRecord, BlobId, CanonicalCatalog,
    CanonicalPublicationHead, CatalogGeneration, ChecksummedPublicationHead, LayoutDecodePolicy,
    LayoutEntryLimit, RetentionClosureLimits, RetentionClosureVerificationError, RetentionRoot,
    SegmentRecordIdentity, VerifiedRetentionClosure, verify_retention_closure,
};

use super::{
    ONE_ZERO_BLOB, maximum_policy,
    memory_stage::segment_bytes,
    one_zero_bundle::{root_with_limits, verify_fixture},
    support::{layout_record_bytes, require_error},
};

const CHUNK_CATALOG_HEX: &str =
    include_str!("../../conformance/segment-store/v1/one-zero-catalog.hex");
const CHUNK_HEAD_HEX: &str = include_str!("../../conformance/segment-store/v1/one-zero-head.hex");
const CHUNK_SEGMENT_HEX: &str =
    include_str!("../../conformance/segment-store/v1/one-zero-segment.hex");

#[test]
fn missing_layout_is_an_exact_first_scheduled_member_refusal() -> Result<(), Box<dyn Error>> {
    let root = root_with_limits(RetentionClosureLimits::new(2, 2, 220, 509)?, None)?;
    let error = require_error(
        verify_fixture(&root, CHUNK_SEGMENT_HEX, CHUNK_CATALOG_HEX, CHUNK_HEAD_HEX)?,
        "chunk-only catalog unexpectedly satisfied a retained layout",
    )?;
    let expected = root
        .anchors()
        .first()
        .copied()
        .ok_or("retention root omits its required anchor")?
        .layout_id();

    assert!(matches!(
        error,
        RetentionClosureVerificationError::MissingMember {
            identity: SegmentRecordIdentity::Layout(layout)
        } if layout == expected
    ));
    Ok(())
}

#[test]
fn missing_chunk_is_an_exact_logical_occurrence_refusal() -> Result<(), Box<dyn Error>> {
    let root = root_with_limits(RetentionClosureLimits::new(2, 2, 220, 509)?, None)?;
    let error = require_error(
        verify_layout_only(&root)?,
        "layout-only catalog unexpectedly reconstructed a retained blob",
    )?;
    let expected = expected_chunk_identity()?;

    assert!(matches!(
        error,
        RetentionClosureVerificationError::MissingMember { identity } if identity == expected
    ));
    Ok(())
}

#[test]
fn anchor_target_mismatch_precedes_chunk_traversal() -> Result<(), Box<dyn Error>> {
    let expected = BlobId::hash_bytes(b"adversarial anchor")?;
    let root = root_with_limits(RetentionClosureLimits::new(2, 2, 220, 509)?, Some(expected))?;
    let error = require_error(
        verify_layout_only(&root)?,
        "mismatched anchor target unexpectedly verified",
    )?;
    let observed: BlobId = ONE_ZERO_BLOB.parse()?;

    assert!(matches!(
        error,
        RetentionClosureVerificationError::AnchorTargetMismatch {
            expected: actual_expected,
            observed: actual_observed,
            ..
        } if actual_expected == expected && actual_observed == observed
    ));
    Ok(())
}

fn verify_layout_only(
    root: &RetentionRoot,
) -> Result<Result<VerifiedRetentionClosure, RetentionClosureVerificationError>, Box<dyn Error>> {
    let layout = one_zero_layout()?;
    let canonical_layout = layout.encode_record()?;
    let records = [AdmittedSegmentRecord::for_layout(&canonical_layout)?];
    let segment_bytes = segment_bytes(&records)?;
    let segment = AdmittedSegment::decode(&segment_bytes, maximum_policy())?;
    let segments = [segment];
    let catalog = CanonicalCatalog::from_segments(CatalogGeneration::new(1)?, None, &segments)?;
    let head = CanonicalPublicationHead::for_catalog(catalog.checksummed());
    let admitted = catalog.checksummed().admit(&segments)?;
    let snapshot = ChecksummedPublicationHead::decode(head.encoded())?.admit(admitted)?;
    Ok(verify_retention_closure(root, &snapshot))
}

fn expected_chunk_identity() -> Result<SegmentRecordIdentity, Box<dyn Error>> {
    let layout = one_zero_layout()?;
    let entry = layout
        .entries()
        .first()
        .copied()
        .ok_or("one-zero layout omits its chunk entry")?;
    Ok(SegmentRecordIdentity::Chunk(entry.chunk_id()))
}

fn one_zero_layout() -> Result<AdmittedLayout, Box<dyn Error>> {
    Ok(AdmittedLayout::decode_record(
        &layout_record_bytes("one-zero")?,
        LayoutDecodePolicy::new(LayoutEntryLimit::MAXIMUM),
    )?)
}
