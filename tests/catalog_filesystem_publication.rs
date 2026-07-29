//! Filesystem-backed catalog publication laws.

#[path = "catalog_filesystem_publication/refusal_laws.rs"]
mod refusal_laws;
#[path = "segment_filesystem_stage/sandbox.rs"]
pub mod sandbox;
mod support;

use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use keep::{
    AdmittedSegment, AdmittedSegmentRecord, CanonicalCatalog, CatalogGeneration,
    CatalogPublicationExpectation, CatalogRestartByteLimit, CatalogRestartPolicy, ClosedSegment,
    FilesystemCatalogPublisher, FilesystemCatalogSnapshot, FilesystemWriterLock, LayoutEntryLimit,
    SegmentPublication, SegmentReadPolicy, SegmentRecordLimit, StagedSegment,
    publish_catalog_generation,
};
use sandbox::TestDirectory;
use support::decode_hex;

const SEGMENT_HEX: &str = include_str!("../conformance/segment-store/v1/one-zero-segment.hex");
const EMPTY_SEGMENT_HEX: &str = include_str!("../conformance/segment-store/v1/empty-segment.hex");
const CATALOG_HEX: &str = include_str!("../conformance/segment-store/v1/one-zero-catalog.hex");
const HEAD_HEX: &str = include_str!("../conformance/segment-store/v1/one-zero-head.hex");
const CATALOG_DIGEST: &str = "04b82519b0399baefd0b9c0f32a871052e4c47e3a00226ab03b21661470f7320";
const SEGMENT_DIGEST: &str = "b7542dced2ab770894a14d1d04b066e3a899942602c5986d35ba6df6c1a35cfc";
const RETAINED_SEGMENT_LIMIT: u64 = 1_048_576;

type StagedFixture = (ClosedSegment, Vec<u8>);

#[test]
fn successful_publication_materializes_only_the_exact_durable_view() -> Result<(), Box<dyn Error>> {
    let store = StoreFixture::create("catalog-filesystem-success")?;
    let lock = FilesystemWriterLock::try_acquire(store.path())?;
    let mut publisher = FilesystemCatalogPublisher::open(lock, restart_policy()?)?;
    let (closed, segment_bytes) = stage_one_zero(&publisher, &store)?;
    assert_eq!(segment_bytes, fixture(SEGMENT_HEX)?);
    let segment = AdmittedSegment::decode(&segment_bytes, maximum_segment_policy())?;
    let segments = [segment];
    let catalog = CanonicalCatalog::from_segments(CatalogGeneration::new(1)?, None, &segments)?;
    let selection = SegmentPublication::one(closed, &segments[0])?;

    let receipt = publish_catalog_generation(
        &mut publisher,
        CatalogPublicationExpectation::uninitialized(),
        selection,
        &catalog,
        &segments,
    )?;
    drop(publisher);

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

struct StoreFixture {
    sandbox: TestDirectory,
    catalog_path: PathBuf,
    segment_path: PathBuf,
}

impl StoreFixture {
    fn create(name: &str) -> Result<Self, Box<dyn Error>> {
        let sandbox = TestDirectory::create(name)?;
        fs::write(sandbox.path().join("writer.lock"), [])?;
        let staging = sandbox.path().join("staging");
        let segments = sandbox.path().join("segments");
        let catalogs = sandbox.path().join("catalogs");
        fs::create_dir(&staging)?;
        fs::create_dir(&segments)?;
        fs::create_dir(&catalogs)?;
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

fn restart_policy() -> Result<CatalogRestartPolicy, Box<dyn Error>> {
    Ok(CatalogRestartPolicy::new(
        maximum_segment_policy(),
        CatalogRestartByteLimit::new(RETAINED_SEGMENT_LIMIT)?,
    ))
}

fn stage_one_zero(
    publisher: &FilesystemCatalogPublisher,
    store: &StoreFixture,
) -> Result<StagedFixture, Box<dyn Error>> {
    let stage = publisher.create_segment_stage()?;
    let record = AdmittedSegmentRecord::for_chunk(&[0])?;
    let closed = StagedSegment::begin(stage, SegmentRecordLimit::MAXIMUM)?
        .append(record)?
        .seal()?
        .close();
    let bytes = fs::read(store.staging().join("current.seg"))?;
    Ok((closed, bytes))
}

const fn maximum_segment_policy() -> SegmentReadPolicy {
    SegmentReadPolicy::new(SegmentRecordLimit::MAXIMUM, LayoutEntryLimit::MAXIMUM)
}

fn fixture(hex: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    decode_hex(hex.strip_suffix('\n').ok_or("fixture must end in one LF")?).map_err(Into::into)
}
