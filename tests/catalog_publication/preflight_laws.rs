//! Publication preflight and current-state transition laws.

use std::error::Error;

use keep::{
    AdmittedSegment, CanonicalCatalog, CanonicalPublicationHead, CatalogGeneration,
    CatalogPublicationError, CatalogPublicationExpectation, CatalogPublicationOutcome,
    CatalogPublicationPhase, CatalogTransitionError, ChecksummedPublicationHead,
    SegmentPublication, publish_catalog_generation,
};

use super::recording_storage::RecordingStorage;
use super::segment_selection;
use super::{EMPTY_SEGMENT_HEX, SEGMENT_HEX, fixture, maximum_policy, publication_fixture};
use crate::support::require_error;

#[test]
fn staged_segment_must_belong_to_the_admitted_set_before_io() -> Result<(), Box<dyn Error>> {
    let bytes = fixture(SEGMENT_HEX)?;
    let publication = publication_fixture(&bytes)?;
    let staged_bytes = fixture(EMPTY_SEGMENT_HEX)?;
    let staged = AdmittedSegment::decode(&staged_bytes, maximum_policy())?;
    let expected = staged.digest();
    let selection = segment_selection::for_segment(&staged)?;
    let mut storage = RecordingStorage::succeeding();
    let error = require_error(
        publish_catalog_generation(
            &mut storage,
            CatalogPublicationExpectation::uninitialized(),
            selection,
            &publication.catalog,
            &publication.segments,
        ),
        "unadmitted staged segment reached publication",
    )?;

    assert!(matches!(
        error,
        CatalogPublicationError::StagedSegmentNotAdmitted { segment_digest }
            if segment_digest == expected
    ));
    assert!(storage.observed().is_empty());
    Ok(())
}

#[test]
fn catalog_location_refusal_precedes_every_storage_call() -> Result<(), Box<dyn Error>> {
    let bytes = fixture(SEGMENT_HEX)?;
    let publication = publication_fixture(&bytes)?;
    let mut storage = RecordingStorage::succeeding();
    let error = require_error(
        publish_catalog_generation(
            &mut storage,
            CatalogPublicationExpectation::uninitialized(),
            SegmentPublication::none(),
            &publication.catalog,
            &[],
        ),
        "catalog with a missing segment reached publication",
    )?;

    assert!(matches!(
        error,
        CatalogPublicationError::CatalogAdmission { .. }
    ));
    assert!(storage.observed().is_empty());
    Ok(())
}

#[test]
fn already_published_retry_only_reverifies_and_synchronizes_root() -> Result<(), Box<dyn Error>> {
    let bytes = fixture(SEGMENT_HEX)?;
    let publication = publication_fixture(&bytes)?;
    let mut storage = RecordingStorage::already_published();
    let receipt = publish_catalog_generation(
        &mut storage,
        CatalogPublicationExpectation::uninitialized(),
        SegmentPublication::none(),
        &publication.catalog,
        &publication.segments,
    )?;

    assert_eq!(
        receipt.outcome(),
        CatalogPublicationOutcome::AlreadyPublished
    );
    assert_eq!(
        storage.observed(),
        &[
            CatalogPublicationPhase::VerifyCurrent,
            CatalogPublicationPhase::SynchronizeRoot,
        ]
    );
    Ok(())
}

#[test]
fn current_snapshot_requires_and_admits_only_its_exact_successor() -> Result<(), Box<dyn Error>> {
    let bytes = fixture(SEGMENT_HEX)?;
    let publication = publication_fixture(&bytes)?;
    let current_catalog = publication.catalog.checksummed();
    let current_head = CanonicalPublicationHead::for_catalog(current_catalog);
    let checked_head = ChecksummedPublicationHead::decode(current_head.encoded())?;
    let admitted = current_catalog.admit(&publication.segments)?;
    let current = checked_head.admit(admitted)?;
    let expectation = CatalogPublicationExpectation::successor_of(&current);
    let mut storage = RecordingStorage::succeeding();

    let stale = require_error(
        publish_catalog_generation(
            &mut storage,
            expectation,
            SegmentPublication::none(),
            &publication.catalog,
            &publication.segments,
        ),
        "current generation was republished as its own successor",
    )?;
    assert!(matches!(
        stale,
        CatalogPublicationError::Transition {
            source: CatalogTransitionError::Generation {
                expected,
                observed,
            },
        } if expected.get() == 2 && observed.get() == 1
    ));
    assert!(storage.observed().is_empty());

    let successor = CanonicalCatalog::from_segments(
        CatalogGeneration::new(2)?,
        Some(current.catalog_digest()),
        &publication.segments,
    )?;
    let wrong_initial = require_error(
        publish_catalog_generation(
            &mut storage,
            CatalogPublicationExpectation::uninitialized(),
            SegmentPublication::none(),
            &successor,
            &publication.segments,
        ),
        "generation 2 initialized an uninitialized store",
    )?;
    assert!(matches!(
        wrong_initial,
        CatalogPublicationError::InitialGeneration { observed } if observed.get() == 2
    ));
    assert!(storage.observed().is_empty());

    let receipt = publish_catalog_generation(
        &mut storage,
        expectation,
        SegmentPublication::none(),
        &successor,
        &publication.segments,
    )?;
    assert_eq!(receipt.generation().get(), 2);
    Ok(())
}
