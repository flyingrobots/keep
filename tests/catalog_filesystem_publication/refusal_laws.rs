//! Filesystem publication conflict, staleness, and recovery-refusal laws.

use std::error::Error;
use std::fs;

use keep::{
    AdmittedSegment, CanonicalCatalog, CatalogGeneration, CatalogPublicationError,
    CatalogPublicationExpectation, CatalogPublicationPhase, FilesystemCatalogPublicationError,
    FilesystemCatalogPublisher, FilesystemWriterLock, SegmentPublication,
    publish_catalog_generation,
};

use super::{
    EMPTY_SEGMENT_HEX, StoreFixture, fixture, maximum_segment_policy, restart_policy,
    stage_one_zero,
};
use crate::support::require_error;

#[test]
fn conflicting_immutable_pool_bytes_refuse_before_visibility() -> Result<(), Box<dyn Error>> {
    let store = StoreFixture::create("catalog-filesystem-conflict")?;
    let (closed, segment_bytes) = stage_one_zero(&store)?;
    fs::write(store.segment_path(), fixture(EMPTY_SEGMENT_HEX)?)?;
    let segment = AdmittedSegment::decode(&segment_bytes, maximum_segment_policy())?;
    let segments = [segment];
    let catalog = CanonicalCatalog::from_segments(CatalogGeneration::new(1)?, None, &segments)?;
    let lock = FilesystemWriterLock::try_acquire(store.path())?;
    let mut publisher = FilesystemCatalogPublisher::open(lock, restart_policy()?)?;
    let selection = SegmentPublication::one(closed, &segments[0])?;

    let error = require_error(
        publish_catalog_generation(
            &mut publisher,
            CatalogPublicationExpectation::uninitialized(),
            selection,
            &catalog,
            &segments,
        ),
        "conflicting immutable segment was published",
    )?;
    drop(publisher);

    assert!(matches!(
        error,
        CatalogPublicationError::Storage {
            phase: CatalogPublicationPhase::VerifySegmentPool,
            ..
        }
    ));
    assert!(!store.path().join("HEAD").exists());
    assert!(store.staging().join("current.seg").exists());
    store.remove()
}

#[test]
fn stale_current_head_refuses_before_creating_catalog_state() -> Result<(), Box<dyn Error>> {
    let store = StoreFixture::create("catalog-filesystem-stale")?;
    let (closed, segment_bytes) = stage_one_zero(&store)?;
    let segment = AdmittedSegment::decode(&segment_bytes, maximum_segment_policy())?;
    let segments = [segment];
    let catalog = CanonicalCatalog::from_segments(CatalogGeneration::new(1)?, None, &segments)?;
    let lock = FilesystemWriterLock::try_acquire(store.path())?;
    let mut publisher = FilesystemCatalogPublisher::open(lock, restart_policy()?)?;
    let selection = SegmentPublication::one(closed, &segments[0])?;
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
    let error = require_error(
        publish_catalog_generation(
            &mut publisher,
            CatalogPublicationExpectation::uninitialized(),
            SegmentPublication::none(),
            &catalog,
            &segments,
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
    let (closed, segment_bytes) = stage_one_zero(&store)?;
    let segment = AdmittedSegment::decode(&segment_bytes, maximum_segment_policy())?;
    let segments = [segment];
    let catalog = CanonicalCatalog::from_segments(CatalogGeneration::new(1)?, None, &segments)?;
    let lock = FilesystemWriterLock::try_acquire(store.path())?;
    let mut publisher = FilesystemCatalogPublisher::open(lock, restart_policy()?)?;
    let selection = SegmentPublication::one(closed, &segments[0])?;

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
