//! Catalog-generation publication ordering and fault laws.

#[path = "catalog_publication/closed_stage_laws.rs"]
mod closed_stage_laws;
#[path = "catalog_publication/preflight_laws.rs"]
mod preflight_laws;
#[path = "catalog_publication/recording_storage.rs"]
pub mod recording_storage;
#[path = "catalog_publication/segment_selection.rs"]
pub mod segment_selection;
mod support;

use std::error::Error;

use keep::{
    AdmittedSegment, CanonicalCatalog, CatalogGeneration, CatalogPublicationError,
    CatalogPublicationExpectation, CatalogPublicationPhase, LayoutEntryLimit, SegmentPublication,
    SegmentReadPolicy, SegmentRecordLimit, publish_catalog_generation,
};
use recording_storage::{EXPECTED_WITH_SEGMENT, RecordingStorage};
use support::{decode_hex, require_error};

const SEGMENT_HEX: &str = include_str!("../conformance/segment-store/v1/one-zero-segment.hex");
const EMPTY_SEGMENT_HEX: &str = include_str!("../conformance/segment-store/v1/empty-segment.hex");

#[test]
fn one_generation_executes_every_durability_transition_in_order() -> Result<(), Box<dyn Error>> {
    let bytes = fixture(SEGMENT_HEX)?;
    let fixture = publication_fixture(&bytes)?;
    let staged = fixture.segments.first().ok_or("missing staged segment")?;
    let selection = segment_selection::for_segment(staged)?;
    let mut storage = RecordingStorage::succeeding();

    let receipt = publish_catalog_generation(
        &mut storage,
        CatalogPublicationExpectation::uninitialized(),
        selection,
        &fixture.catalog,
        &fixture.segments,
    )?;

    assert_eq!(storage.observed(), EXPECTED_WITH_SEGMENT);
    assert_eq!(receipt.generation().get(), 1);
    assert_eq!(
        receipt.catalog_digest(),
        fixture.catalog.checksummed().digest()
    );
    Ok(())
}

#[test]
fn every_publication_fault_stops_at_its_exact_phase() -> Result<(), Box<dyn Error>> {
    let bytes = fixture(SEGMENT_HEX)?;
    let fixture = publication_fixture(&bytes)?;
    let staged = fixture.segments.first().ok_or("missing staged segment")?;

    for failing_phase in EXPECTED_WITH_SEGMENT {
        let selection = segment_selection::for_segment(staged)?;
        let mut storage = RecordingStorage::failing_at(*failing_phase);
        let error = require_error(
            publish_catalog_generation(
                &mut storage,
                CatalogPublicationExpectation::uninitialized(),
                selection,
                &fixture.catalog,
                &fixture.segments,
            ),
            "faulting publication returned a receipt",
        )?;

        assert!(matches!(
            error,
            CatalogPublicationError::Storage { phase, .. } if phase == *failing_phase
        ));
        assert_eq!(storage.observed().last(), Some(failing_phase));
        assert_eq!(
            storage.observed().len(),
            expected_prefix_length(*failing_phase)?
        );
    }
    Ok(())
}

#[test]
fn catalog_only_publication_skips_segment_transitions() -> Result<(), Box<dyn Error>> {
    let bytes = fixture(SEGMENT_HEX)?;
    let fixture = publication_fixture(&bytes)?;
    let mut storage = RecordingStorage::succeeding();

    let _receipt = publish_catalog_generation(
        &mut storage,
        CatalogPublicationExpectation::uninitialized(),
        SegmentPublication::none(),
        &fixture.catalog,
        &fixture.segments,
    )?;

    assert_eq!(
        storage.observed().get(1),
        Some(&CatalogPublicationPhase::CreateCatalogStage)
    );
    assert_eq!(
        storage.observed().len(),
        EXPECTED_WITH_SEGMENT
            .len()
            .checked_sub(5)
            .ok_or("publication phase count underflowed")?
    );
    Ok(())
}

struct PublicationFixture<'a> {
    segments: [AdmittedSegment<'a>; 1],
    catalog: CanonicalCatalog,
}

fn publication_fixture(bytes: &[u8]) -> Result<PublicationFixture<'_>, Box<dyn Error>> {
    let segments = [AdmittedSegment::decode(bytes, maximum_policy())?];
    let catalog = CanonicalCatalog::from_segments(CatalogGeneration::new(1)?, None, &segments)?;
    Ok(PublicationFixture { segments, catalog })
}

fn expected_prefix_length(phase: CatalogPublicationPhase) -> Result<usize, Box<dyn Error>> {
    EXPECTED_WITH_SEGMENT
        .iter()
        .position(|candidate| *candidate == phase)
        .and_then(|index| index.checked_add(1))
        .ok_or_else(|| "missing expected publication phase".into())
}

const fn maximum_policy() -> SegmentReadPolicy {
    SegmentReadPolicy::new(SegmentRecordLimit::MAXIMUM, LayoutEntryLimit::MAXIMUM)
}

fn fixture(hex: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    decode_hex(hex.strip_suffix('\n').ok_or("fixture must end in one LF")?).map_err(Into::into)
}
