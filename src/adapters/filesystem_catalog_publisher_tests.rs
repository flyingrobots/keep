//! Filesystem-backed catalog publication laws.

#[path = "../../tests/catalog_filesystem_publication/authority_laws.rs"]
mod authority_laws;
#[path = "../../tests/catalog_filesystem_publication/directory_laws.rs"]
mod directory_laws;
#[path = "../../tests/catalog_filesystem_publication/initialization_laws.rs"]
mod initialization_laws;
#[path = "../../tests/catalog_filesystem_publication/refusal_laws.rs"]
mod refusal_laws;

use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use super::filesystem_test_sandbox::TestDirectory;
use super::test_support::decode_hex;
use crate::{
    AdmittedSegment, AdmittedSegmentRecord, CanonicalCatalog, CatalogGeneration,
    CatalogPublicationError, CatalogPublicationExpectation, CatalogPublicationOutcome,
    CatalogRestartByteLimit, CatalogRestartPolicy, FilesystemCatalogSnapshot,
    FilesystemPlatformAdmissionError, FilesystemSegmentStage, FilesystemVersionTwoAdmission,
    FilesystemVersionTwoCatalogPublisher, LayoutEntryLimit, RetainedCatalogPublicationError,
    SealedSegment, SegmentPublication, SegmentReadPolicy, SegmentRecordLimit, StagedSegment,
    publish_retained_catalog_generation,
};

const SEGMENT_HEX: &str = include_str!("../../conformance/segment-store/v1/one-zero-segment.hex");
const EMPTY_SEGMENT_HEX: &str =
    include_str!("../../conformance/segment-store/v1/empty-segment.hex");
const CATALOG_HEX: &str = include_str!("../../conformance/segment-store/v1/one-zero-catalog.hex");
const HEAD_HEX: &str = include_str!("../../conformance/segment-store/v1/one-zero-head.hex");
const CATALOG_DIGEST: &str = "04b82519b0399baefd0b9c0f32a871052e4c47e3a00226ab03b21661470f7320";
const SEGMENT_DIGEST: &str = "b7542dced2ab770894a14d1d04b066e3a899942602c5986d35ba6df6c1a35cfc";
const RETAINED_SEGMENT_LIMIT: u64 = 1_048_576;
type StagedFixture<'publisher> = (SealedSegment<FilesystemSegmentStage<'publisher>>, Vec<u8>);

#[test]
fn successful_publication_materializes_only_the_exact_durable_view() -> Result<(), Box<dyn Error>> {
    let store = StoreFixture::create("catalog-filesystem-success")?;
    let mut publisher = open_publisher(&store)?;
    let (sealed, segment_bytes) = stage_one_zero(&publisher, &store)?;
    assert_eq!(segment_bytes, fixture(SEGMENT_HEX)?);
    let segment = AdmittedSegment::decode(&segment_bytes, maximum_segment_policy())?;
    let segments = [segment];
    let catalog = CanonicalCatalog::from_segments(CatalogGeneration::new(1)?, None, &segments)?;
    let selection = publisher.select_segment(sealed, &segments[0])?;

    let receipt = publish_retained_catalog_generation(
        &mut publisher,
        CatalogPublicationExpectation::uninitialized(),
        selection,
        &catalog,
        &segments,
    )?;
    drop(publisher);

    assert_eq!(receipt.outcome(), CatalogPublicationOutcome::Published);
    assert_eq!(receipt.generation().get(), 1);
    assert_eq!(fs::read(store.path().join("HEAD"))?, fixture(HEAD_HEX)?);
    assert_eq!(fs::read(store.catalog_path())?, fixture(CATALOG_HEX)?);
    assert_eq!(fs::read(store.segment_path())?, segment_bytes);
    assert!(!store.staging().join("current.seg").exists());
    assert!(!store.staging().join("current.cat").exists());
    assert!(!store.path().join("head.next").exists());
    let loaded = FilesystemCatalogSnapshot::load(store.path(), restart_policy()?)?;
    assert_eq!(loaded.catalog_digest(), receipt.catalog_digest());
    store.remove()
}

#[test]
fn durable_publication_retry_returns_the_same_synchronized_receipt() -> Result<(), Box<dyn Error>> {
    let store = StoreFixture::create("catalog-filesystem-retry")?;
    let mut publisher = open_publisher(&store)?;
    let (sealed, segment_bytes) = stage_one_zero(&publisher, &store)?;
    let segment = AdmittedSegment::decode(&segment_bytes, maximum_segment_policy())?;
    let segments = [segment];
    let catalog = CanonicalCatalog::from_segments(CatalogGeneration::new(1)?, None, &segments)?;
    let selection = publisher.select_segment(sealed, &segments[0])?;
    let first = publish_retained_catalog_generation(
        &mut publisher,
        CatalogPublicationExpectation::uninitialized(),
        selection,
        &catalog,
        &segments,
    )?;
    drop(publisher);

    let mut publisher = open_publisher(&store)?;
    let retry = publish_retained_catalog_generation(
        &mut publisher,
        CatalogPublicationExpectation::uninitialized(),
        SegmentPublication::none(),
        &catalog,
        &segments,
    )?;

    assert_eq!(retry.generation(), first.generation());
    assert_eq!(retry.catalog_digest(), first.catalog_digest());
    assert_eq!(retry.outcome(), CatalogPublicationOutcome::AlreadyPublished);
    drop(publisher);
    store.remove()
}

struct StoreFixture {
    sandbox: TestDirectory,
    catalog_path: PathBuf,
    segment_path: PathBuf,
}

impl StoreFixture {
    fn create(name: &str) -> Result<Self, Box<dyn Error>> {
        let sandbox = TestDirectory::create(name)?;
        drop(
            FilesystemVersionTwoAdmission::initialize_unchecked_for_repository_tasks(
                sandbox.path(),
            )?,
        );
        let segments = sandbox.path().join("segments");
        let catalogs = sandbox.path().join("catalogs");
        Ok(Self {
            catalog_path: catalogs.join(format!("0000000000000001-{CATALOG_DIGEST}.cat")),
            segment_path: segments.join(format!("{SEGMENT_DIGEST}.seg")),
            sandbox,
        })
    }

    fn path(&self) -> &Path {
        self.sandbox.path()
    }

    fn staging(&self) -> PathBuf {
        self.path().join("staging")
    }

    fn catalog_path(&self) -> &Path {
        &self.catalog_path
    }

    fn segment_path(&self) -> &Path {
        &self.segment_path
    }

    fn remove(self) -> Result<(), Box<dyn Error>> {
        self.sandbox.remove().map_err(Into::into)
    }
}

/// Reacquires single-format writer authority over the initialized store.
fn admit_writer(
    store: &StoreFixture,
) -> Result<FilesystemVersionTwoAdmission, FilesystemPlatformAdmissionError> {
    FilesystemVersionTwoAdmission::reopen_unchecked_for_repository_tasks(store.path())
}

/// Reopens the initialized store and pins its single-format writer.
fn open_publisher(
    store: &StoreFixture,
) -> Result<FilesystemVersionTwoCatalogPublisher, Box<dyn Error>> {
    Ok(FilesystemVersionTwoCatalogPublisher::open(
        admit_writer(store)?,
        restart_policy()?,
    )?)
}

/// The catalog-protocol refusal inside one gated publication failure.
fn catalog_refusal(
    error: RetainedCatalogPublicationError,
) -> Result<CatalogPublicationError, Box<dyn Error>> {
    match error {
        RetainedCatalogPublicationError::Catalog { source } => Ok(source),
        other => Err(format!("refusal outside the catalog protocol: {other}").into()),
    }
}

fn restart_policy() -> Result<CatalogRestartPolicy, Box<dyn Error>> {
    Ok(CatalogRestartPolicy::new(
        maximum_segment_policy(),
        CatalogRestartByteLimit::new(RETAINED_SEGMENT_LIMIT)?,
    ))
}

fn stage_one_zero<'publisher>(
    publisher: &'publisher FilesystemVersionTwoCatalogPublisher,
    store: &StoreFixture,
) -> Result<StagedFixture<'publisher>, Box<dyn Error>> {
    let stage = publisher.create_segment_stage()?;
    let record = AdmittedSegmentRecord::for_chunk(&[0])?;
    let sealed = StagedSegment::begin(stage, SegmentRecordLimit::MAXIMUM)?
        .append(record)?
        .seal()?;
    let bytes = fs::read(store.staging().join("current.seg"))?;
    Ok((sealed, bytes))
}

const fn maximum_segment_policy() -> SegmentReadPolicy {
    SegmentReadPolicy::new(SegmentRecordLimit::MAXIMUM, LayoutEntryLimit::MAXIMUM)
}

fn fixture(hex: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    decode_hex(hex.strip_suffix('\n').ok_or("fixture must end in one LF")?).map_err(Into::into)
}

#[test]
fn publisher_drop_closes_writable_stages_before_releasing_writer_authority()
-> Result<(), Box<dyn Error>> {
    let source = include_str!("filesystem_version_two_catalog_publisher.rs");
    let stages = source
        .find("pub(super) stages: FilesystemVersionTwoCatalogStages,")
        .ok_or("publisher must retain the gated catalog stages")?;
    let writer_lock = source
        .find("    _lock: FilesystemWriterLock,")
        .ok_or("publisher must retain writer authority")?;
    let shared = include_str!("filesystem_catalog_stages.rs");
    let owner = shared
        .find("pub(super) struct FilesystemCatalogStages")
        .ok_or("shared stages must be one owned type")?;
    let catalog_stage = shared
        .find("pub(super) catalog_stage:")
        .ok_or("shared stages must retain the catalog stage")?;
    let head_stage = shared
        .find("pub(super) head_stage:")
        .ok_or("shared stages must retain the head stage")?;

    assert!(stages < writer_lock);
    assert!(owner < catalog_stage && owner < head_stage);
    assert!(!shared.contains("FilesystemWriterLock"));
    Ok(())
}
