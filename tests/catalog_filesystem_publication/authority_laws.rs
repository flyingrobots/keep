use std::error::Error;
use std::fs;
use std::io::{self, Write};

use keep::{
    AdmittedSegment, CanonicalCatalog, CatalogGeneration, CatalogPublicationError,
    CatalogPublicationExpectation, CatalogPublicationPhase, FilesystemCatalogPublicationError,
    FilesystemCatalogPublisher, FilesystemWriterLock, SegmentPublication, SegmentRecordLimit,
    SegmentStage, StagedSegment, publish_catalog_generation,
};

use super::{
    SEGMENT_HEX, StoreFixture, fixture, maximum_segment_policy, restart_policy, stage_one_zero,
};

#[test]
fn metadata_equivalent_external_stage_cannot_authorize_publication() -> Result<(), Box<dyn Error>> {
    let store = StoreFixture::create("catalog-filesystem-stage-authority")?;
    let segment_bytes = fixture(SEGMENT_HEX)?;
    fs::write(store.staging().join("current.seg"), &segment_bytes)?;
    let segment = AdmittedSegment::decode(&segment_bytes, maximum_segment_policy())?;
    let mut staged = StagedSegment::begin(MemoryStage::default(), SegmentRecordLimit::MAXIMUM)?;
    for record in segment.records() {
        staged = staged.append(record?)?;
    }
    let segments = [segment];
    let selection = SegmentPublication::one(staged.seal()?.close(), &segments[0])?;
    let catalog = CanonicalCatalog::from_segments(CatalogGeneration::new(1)?, None, &segments)?;
    let lock = FilesystemWriterLock::try_acquire(store.path())?;
    let mut publisher = FilesystemCatalogPublisher::open(lock, restart_policy()?)?;

    let Err(error) = publish_catalog_generation(
        &mut publisher,
        CatalogPublicationExpectation::uninitialized(),
        selection,
        &catalog,
        &segments,
    ) else {
        return Err("external closed-stage metadata authorized publication".into());
    };
    let CatalogPublicationError::Storage { phase, source } = error else {
        return Err("external stage reached the wrong refusal boundary".into());
    };

    assert_eq!(phase, CatalogPublicationPhase::VerifyCurrent);
    assert_eq!(source.kind(), io::ErrorKind::InvalidData);
    assert!(matches!(
        source
            .get_ref()
            .and_then(|error| error.downcast_ref::<FilesystemCatalogPublicationError>()),
        Some(FilesystemCatalogPublicationError::SegmentAuthorityRequired)
    ));
    drop(publisher);
    store.remove()
}

#[test]
fn one_publisher_cannot_select_another_publishers_stage() -> Result<(), Box<dyn Error>> {
    let first_store = StoreFixture::create("catalog-filesystem-stage-owner")?;
    let second_store = StoreFixture::create("catalog-filesystem-stage-substitute")?;
    let first_lock = FilesystemWriterLock::try_acquire(first_store.path())?;
    let first_publisher = FilesystemCatalogPublisher::open(first_lock, restart_policy()?)?;
    let second_lock = FilesystemWriterLock::try_acquire(second_store.path())?;
    let second_publisher = FilesystemCatalogPublisher::open(second_lock, restart_policy()?)?;
    let (sealed, segment_bytes) = stage_one_zero(&first_publisher, &first_store)?;
    let segment = AdmittedSegment::decode(&segment_bytes, maximum_segment_policy())?;

    let Err(error) = second_publisher.select_segment(sealed, &segment) else {
        return Err("publisher selected another publisher's sealed stage".into());
    };

    assert!(matches!(
        error,
        keep::SegmentPublicationError::PublisherAuthority
    ));
    drop(first_publisher);
    drop(second_publisher);
    first_store.remove()?;
    second_store.remove()
}

#[derive(Default)]
struct MemoryStage {
    bytes: Vec<u8>,
}

impl Write for MemoryStage {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl SegmentStage for MemoryStage {
    fn synchronize(&mut self) -> io::Result<()> {
        Ok(())
    }
}
