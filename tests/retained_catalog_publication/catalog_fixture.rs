//! Current and candidate catalogs for retained-closure publication laws.

use std::error::Error;

use keep::{
    AdmittedLayout, AdmittedSegment, AdmittedSegmentRecord, CanonicalCatalog,
    CanonicalPublicationHead, CatalogGeneration, CatalogPublicationExpectation,
    ChecksummedPublicationHead, LayoutDecodePolicy, LayoutEntryLimit, SegmentReadPolicy,
    SegmentRecordIdentity, SegmentRecordLimit,
};

use crate::memory_stage::segment_bytes;
use crate::retention_fixture::fixture;
use crate::support::layout_record_bytes;

const BUNDLE_SEGMENT_HEX: &str =
    include_str!("../../conformance/segment-store/v1/one-zero-bundle-segment.hex");
const CHUNK_SEGMENT_HEX: &str =
    include_str!("../../conformance/segment-store/v1/one-zero-segment.hex");

/// Generation-two candidate shape relative to the retained one-zero blob.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Candidate {
    /// Keeps the retained layout and chunk records.
    Preserving,
    /// Keeps the retained chunk but drops its layout.
    DroppedLayout,
    /// Keeps the retained layout but substitutes another chunk for its chunk.
    SubstitutedChunk,
    /// Drops every record.
    Empty,
}

impl Candidate {
    /// Every candidate shape, in a fixed order.
    pub const ALL: [Self; 4] = [
        Self::Preserving,
        Self::DroppedLayout,
        Self::SubstitutedChunk,
        Self::Empty,
    ];
}

/// Inputs to one candidate publication call.
pub struct CandidateCall<'call, 'records> {
    /// Exact current catalog the candidate succeeds.
    pub expectation: CatalogPublicationExpectation,
    /// Canonical generation-two candidate.
    pub catalog: &'call CanonicalCatalog,
    /// Admitted candidate segment set.
    pub segments: &'call [AdmittedSegment<'records>],
}

/// Runs `operation` with one candidate succeeding the retained bundle catalog.
///
/// # Errors
///
/// Returns the exact fixture, segment, catalog, or snapshot refusal.
pub fn with_candidate<T>(
    candidate: Candidate,
    operation: impl FnOnce(CandidateCall<'_, '_>) -> T,
) -> Result<T, Box<dyn Error>> {
    let current_bytes = fixture(BUNDLE_SEGMENT_HEX)?;
    let current_segments = [AdmittedSegment::decode(&current_bytes, maximum_policy())?];
    let current =
        CanonicalCatalog::from_segments(CatalogGeneration::new(1)?, None, &current_segments)?;
    let expectation = expectation_for(&current, &current_segments)?;
    let candidate_bytes = candidate_segment(candidate)?;
    let segments = candidate_bytes
        .as_deref()
        .map(|bytes| AdmittedSegment::decode(bytes, maximum_policy()))
        .transpose()?
        .into_iter()
        .collect::<Vec<_>>();
    let catalog = CanonicalCatalog::from_segments(
        CatalogGeneration::new(2)?,
        Some(current.checksummed().digest()),
        &segments,
    )?;
    Ok(operation(CandidateCall {
        expectation,
        catalog: &catalog,
        segments: &segments,
    }))
}

/// Returns the identity a candidate drops from the retained closure.
///
/// # Errors
///
/// Returns the exact layout fixture refusal or `Preserving`, which drops none.
pub fn dropped_identity(candidate: Candidate) -> Result<SegmentRecordIdentity, Box<dyn Error>> {
    let layout = one_zero_layout()?;
    match candidate {
        Candidate::Preserving => Err("a preserving candidate drops no identity".into()),
        Candidate::DroppedLayout | Candidate::Empty => {
            Ok(SegmentRecordIdentity::Layout(layout.encode_record()?.id()))
        }
        Candidate::SubstitutedChunk => {
            let entry = layout
                .entries()
                .first()
                .copied()
                .ok_or("one-zero layout omits its chunk")?;
            Ok(SegmentRecordIdentity::Chunk(entry.chunk_id()))
        }
    }
}

/// Returns the candidate's only segment, absent for a catalog with no records.
fn candidate_segment(candidate: Candidate) -> Result<Option<Vec<u8>>, Box<dyn Error>> {
    match candidate {
        Candidate::Preserving => fixture(BUNDLE_SEGMENT_HEX).map(Some),
        Candidate::DroppedLayout => fixture(CHUNK_SEGMENT_HEX).map(Some),
        Candidate::Empty => Ok(None),
        Candidate::SubstitutedChunk => {
            let canonical = one_zero_layout()?.encode_record()?;
            let records = [
                AdmittedSegmentRecord::for_chunk(b"substituted chunk")?,
                AdmittedSegmentRecord::for_layout(&canonical)?,
            ];
            Ok(Some(segment_bytes(&records)?))
        }
    }
}

fn expectation_for(
    current: &CanonicalCatalog,
    segments: &[AdmittedSegment<'_>],
) -> Result<CatalogPublicationExpectation, Box<dyn Error>> {
    let head = CanonicalPublicationHead::for_catalog(current.checksummed());
    let admitted = current.checksummed().admit(segments)?;
    let snapshot = ChecksummedPublicationHead::decode(head.encoded())?.admit(admitted)?;
    Ok(CatalogPublicationExpectation::successor_of(&snapshot))
}

fn one_zero_layout() -> Result<AdmittedLayout, Box<dyn Error>> {
    Ok(AdmittedLayout::decode_record(
        &layout_record_bytes("one-zero")?,
        LayoutDecodePolicy::new(LayoutEntryLimit::MAXIMUM),
    )?)
}

const fn maximum_policy() -> SegmentReadPolicy {
    SegmentReadPolicy::new(SegmentRecordLimit::MAXIMUM, LayoutEntryLimit::MAXIMUM)
}
