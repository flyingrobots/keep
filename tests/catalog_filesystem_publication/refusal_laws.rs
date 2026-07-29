//! Filesystem publication conflict, staleness, and recovery-refusal laws.

use std::error::Error;
use std::fs;

use keep::{
    AdmittedSegment, CanonicalCatalog, CatalogGeneration, CatalogPublicationError,
    CatalogPublicationExpectation, CatalogPublicationPhase, CatalogRestartError,
    FilesystemCatalogPublicationError, FilesystemCatalogPublisher, FilesystemCatalogSnapshot,
    FilesystemWriterLock, SegmentPublication, publish_catalog_generation,
};

use super::{
    EMPTY_SEGMENT_HEX, StoreFixture, fixture, maximum_segment_policy, restart_policy,
    stage_one_zero,
};
use crate::support::require_error;

#[test]
fn conflicting_immutable_pool_bytes_refuse_before_visibility() -> Result<(), Box<dyn Error>> {
    let store = StoreFixture::create("catalog-filesystem-conflict")?;
    let lock = FilesystemWriterLock::try_acquire(store.path())?;
    let mut publisher = FilesystemCatalogPublisher::open(lock, restart_policy()?)?;
    let initial_segments = [];
    let initial_catalog =
        CanonicalCatalog::from_segments(CatalogGeneration::new(1)?, None, &initial_segments)?;
    let _initial_receipt = publish_catalog_generation(
        &mut publisher,
        CatalogPublicationExpectation::uninitialized(),
        SegmentPublication::none(),
        &initial_catalog,
        &initial_segments,
    )?;
    let initial_head = fs::read(store.path().join("HEAD"))?;
    let current = FilesystemCatalogSnapshot::load(store.path(), restart_policy()?)?;
    let current_snapshot = current.snapshot()?;
    let expectation = CatalogPublicationExpectation::successor_of(&current_snapshot);
    let (sealed, segment_bytes) = stage_one_zero(&publisher, &store)?;
    let conflicting_bytes = fixture(EMPTY_SEGMENT_HEX)?;
    let conflicting_segment =
        AdmittedSegment::decode(&conflicting_bytes, maximum_segment_policy())?;
    let conflicting_digest = conflicting_segment.digest();
    fs::write(store.segment_path(), conflicting_bytes)?;
    let segment = AdmittedSegment::decode(&segment_bytes, maximum_segment_policy())?;
    let segment_digest = segment.digest();
    let segments = [segment];
    let catalog = CanonicalCatalog::from_segments(
        CatalogGeneration::new(2)?,
        Some(current.catalog_digest()),
        &segments,
    )?;
    let selection = publisher.select_segment(sealed, &segments[0])?;

    let error = require_error(
        publish_catalog_generation(&mut publisher, expectation, selection, &catalog, &segments),
        "conflicting immutable segment was published",
    )?;
    let CatalogPublicationError::Storage { phase, source } = error else {
        return Err("segment conflict reached the wrong refusal boundary".into());
    };
    assert_eq!(phase, CatalogPublicationPhase::VerifySegmentPool);
    assert!(matches!(
        source
            .get_ref()
            .and_then(|error| error.downcast_ref::<CatalogRestartError>()),
        Some(CatalogRestartError::SegmentCoordinate { expected, observed })
            if *expected == segment_digest && *observed == conflicting_digest
    ));
    drop(publisher);
    assert_eq!(fs::read(store.path().join("HEAD"))?, initial_head);
    assert!(store.staging().join("current.seg").exists());
    store.remove()
}

#[test]
fn stale_current_head_refuses_before_creating_catalog_state() -> Result<(), Box<dyn Error>> {
    let store = StoreFixture::create("catalog-filesystem-stale")?;
    let lock = FilesystemWriterLock::try_acquire(store.path())?;
    let mut publisher = FilesystemCatalogPublisher::open(lock, restart_policy()?)?;
    let (sealed, segment_bytes) = stage_one_zero(&publisher, &store)?;
    let segment = AdmittedSegment::decode(&segment_bytes, maximum_segment_policy())?;
    let segments = [segment];
    let catalog = CanonicalCatalog::from_segments(CatalogGeneration::new(1)?, None, &segments)?;
    let selection = publisher.select_segment(sealed, &segments[0])?;
    let _receipt = publish_catalog_generation(
        &mut publisher,
        CatalogPublicationExpectation::uninitialized(),
        selection,
        &catalog,
        &segments,
    )?;
    drop(publisher);

    let stale_candidate = CanonicalCatalog::from_segments(CatalogGeneration::new(1)?, None, &[])?;
    let lock = FilesystemWriterLock::try_acquire(store.path())?;
    let mut publisher = FilesystemCatalogPublisher::open(lock, restart_policy()?)?;
    let error = require_error(
        publish_catalog_generation(
            &mut publisher,
            CatalogPublicationExpectation::uninitialized(),
            SegmentPublication::none(),
            &stale_candidate,
            &[],
        ),
        "stale uninitialized expectation was accepted",
    )?;

    let CatalogPublicationError::Storage { phase, source } = error else {
        return Err("stale expectation returned the wrong error".into());
    };
    assert_eq!(phase, CatalogPublicationPhase::VerifyCurrent);
    assert!(matches!(
        source
            .get_ref()
            .and_then(|error| error.downcast_ref::<FilesystemCatalogPublicationError>()),
        Some(FilesystemCatalogPublicationError::CurrentState {
            expected_generation: None,
            observed_generation: Some(_),
            ..
        })
    ));
    drop(publisher);
    assert!(!store.staging().join("current.cat").exists());
    store.remove()
}

#[test]
fn leftover_next_head_requires_recovery_before_any_mutation() -> Result<(), Box<dyn Error>> {
    let store = StoreFixture::create("catalog-filesystem-next-head")?;
    fs::write(store.path().join("head.next"), [])?;
    let lock = FilesystemWriterLock::try_acquire(store.path())?;
    let mut publisher = FilesystemCatalogPublisher::open(lock, restart_policy()?)?;
    let (sealed, segment_bytes) = stage_one_zero(&publisher, &store)?;
    let segment = AdmittedSegment::decode(&segment_bytes, maximum_segment_policy())?;
    let segments = [segment];
    let catalog = CanonicalCatalog::from_segments(CatalogGeneration::new(1)?, None, &segments)?;
    let selection = publisher.select_segment(sealed, &segments[0])?;

    let error = require_error(
        publish_catalog_generation(
            &mut publisher,
            CatalogPublicationExpectation::uninitialized(),
            selection,
            &catalog,
            &segments,
        ),
        "leftover head.next was silently replaced",
    )?;
    let CatalogPublicationError::Storage { phase, source } = error else {
        return Err("head recovery evidence returned the wrong error".into());
    };
    assert_eq!(phase, CatalogPublicationPhase::VerifyCurrent);
    assert!(matches!(
        source
            .get_ref()
            .and_then(|error| error.downcast_ref::<FilesystemCatalogPublicationError>()),
        Some(FilesystemCatalogPublicationError::HeadRecoveryRequired)
    ));
    drop(publisher);
    assert!(store.path().join("head.next").exists());
    assert!(store.staging().join("current.seg").exists());
    assert!(!store.path().join("HEAD").exists());
    store.remove()
}

#[test]
fn leftover_catalog_stage_refuses_before_segment_pool_mutation() -> Result<(), Box<dyn Error>> {
    let store = StoreFixture::create("catalog-filesystem-catalog-recovery")?;
    let lock = FilesystemWriterLock::try_acquire(store.path())?;
    let mut publisher = FilesystemCatalogPublisher::open(lock, restart_policy()?)?;
    let (sealed, segment_bytes) = stage_one_zero(&publisher, &store)?;
    fs::write(store.staging().join("current.cat"), b"recovery evidence")?;
    let segment = AdmittedSegment::decode(&segment_bytes, maximum_segment_policy())?;
    let segments = [segment];
    let catalog = CanonicalCatalog::from_segments(CatalogGeneration::new(1)?, None, &segments)?;
    let selection = publisher.select_segment(sealed, &segments[0])?;
    let error = require_error(
        publish_catalog_generation(
            &mut publisher,
            CatalogPublicationExpectation::uninitialized(),
            selection,
            &catalog,
            &segments,
        ),
        "leftover catalog stage was discovered after segment publication",
    )?;

    let CatalogPublicationError::Storage { phase, source } = error else {
        return Err("catalog-stage recovery returned the wrong error".into());
    };
    assert_eq!(phase, CatalogPublicationPhase::VerifyCurrent);
    assert!(matches!(
        source
            .get_ref()
            .and_then(|error| error.downcast_ref::<FilesystemCatalogPublicationError>()),
        Some(FilesystemCatalogPublicationError::CatalogRecoveryRequired)
    ));
    assert!(!store.segment_path().exists());
    assert!(store.staging().join("current.seg").exists());
    assert_eq!(
        fs::read(store.staging().join("current.cat"))?,
        b"recovery evidence"
    );
    drop(publisher);
    store.remove()
}

#[test]
fn catalog_only_publication_refuses_a_leftover_segment_stage() -> Result<(), Box<dyn Error>> {
    let store = StoreFixture::create("catalog-filesystem-segment-recovery")?;
    let stage = store.staging().join("current.seg");
    fs::write(&stage, b"recovery evidence")?;
    let catalog = CanonicalCatalog::from_segments(CatalogGeneration::new(1)?, None, &[])?;
    let lock = FilesystemWriterLock::try_acquire(store.path())?;
    let mut publisher = FilesystemCatalogPublisher::open(lock, restart_policy()?)?;
    let error = require_error(
        publish_catalog_generation(
            &mut publisher,
            CatalogPublicationExpectation::uninitialized(),
            SegmentPublication::none(),
            &catalog,
            &[],
        ),
        "catalog-only publication ignored a leftover segment stage",
    )?;

    let CatalogPublicationError::Storage { phase, source } = error else {
        return Err("segment-stage recovery returned the wrong error".into());
    };
    assert_eq!(phase, CatalogPublicationPhase::VerifyCurrent);
    assert!(matches!(
        source
            .get_ref()
            .and_then(|error| error.downcast_ref::<FilesystemCatalogPublicationError>()),
        Some(FilesystemCatalogPublicationError::SegmentRecoveryRequired)
    ));
    assert_eq!(fs::read(stage)?, b"recovery evidence");
    assert!(!store.path().join("HEAD").exists());
    drop(publisher);
    store.remove()
}

#[test]
fn already_published_retry_refuses_a_recreated_segment_stage() -> Result<(), Box<dyn Error>> {
    let store = StoreFixture::create("catalog-filesystem-retry-stage")?;
    let lock = FilesystemWriterLock::try_acquire(store.path())?;
    let mut publisher = FilesystemCatalogPublisher::open(lock, restart_policy()?)?;
    let (sealed, segment_bytes) = stage_one_zero(&publisher, &store)?;
    let segment = AdmittedSegment::decode(&segment_bytes, maximum_segment_policy())?;
    let segments = [segment];
    let catalog = CanonicalCatalog::from_segments(CatalogGeneration::new(1)?, None, &segments)?;
    let selection = publisher.select_segment(sealed, &segments[0])?;
    let _receipt = publish_catalog_generation(
        &mut publisher,
        CatalogPublicationExpectation::uninitialized(),
        selection,
        &catalog,
        &segments,
    )?;
    drop(publisher);

    let lock = FilesystemWriterLock::try_acquire(store.path())?;
    let mut publisher = FilesystemCatalogPublisher::open(lock, restart_policy()?)?;
    let (sealed, retry_bytes) = stage_one_zero(&publisher, &store)?;
    let retry_segment = AdmittedSegment::decode(&retry_bytes, maximum_segment_policy())?;
    let retry_segments = [retry_segment];
    let selection = publisher.select_segment(sealed, &retry_segments[0])?;
    let error = require_error(
        publish_catalog_generation(
            &mut publisher,
            CatalogPublicationExpectation::uninitialized(),
            selection,
            &catalog,
            &retry_segments,
        ),
        "already-published retry ignored a recreated segment stage",
    )?;

    let CatalogPublicationError::Storage { phase, source } = error else {
        return Err("already-published stage returned the wrong error".into());
    };
    assert_eq!(phase, CatalogPublicationPhase::VerifyCurrent);
    assert!(matches!(
        source
            .get_ref()
            .and_then(|error| error.downcast_ref::<FilesystemCatalogPublicationError>()),
        Some(FilesystemCatalogPublicationError::SegmentRecoveryRequired)
    ));
    assert!(store.staging().join("current.seg").exists());
    drop(publisher);
    store.remove()
}
