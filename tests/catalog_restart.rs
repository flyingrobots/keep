//! Published catalog restart-loading laws over a writer-published single-format store.

#![cfg(target_os = "linux")]

#[path = "catalog_restart/refusal_laws.rs"]
mod refusal_laws;
#[path = "segment_filesystem_stage/sandbox.rs"]
pub mod sandbox;
mod support;
#[path = "catalog_restart/verification_laws.rs"]
mod verification_laws;

use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use keep::{
    AdmittedSegment, AdmittedSegmentRecord, CanonicalCatalog, CatalogGeneration,
    CatalogPublicationExpectation, CatalogRestartByteLimit, CatalogRestartPolicy, ChunkId,
    FilesystemCatalogSnapshot, FilesystemVersionTwoAdmission, FilesystemVersionTwoCatalogPublisher,
    LayoutEntryLimit, SegmentReadPolicy, SegmentRecordIdentity, SegmentRecordLimit, StagedSegment,
    publish_retained_catalog_generation,
};
use sandbox::TestDirectory;
use support::decode_hex;

const HEAD_HEX: &str = include_str!("../conformance/segment-store/v1/one-zero-head.hex");
const CATALOG_HEX: &str = include_str!("../conformance/segment-store/v1/one-zero-catalog.hex");
const SEGMENT_HEX: &str = include_str!("../conformance/segment-store/v1/one-zero-segment.hex");
const CATALOG_DIGEST: &str = "04b82519b0399baefd0b9c0f32a871052e4c47e3a00226ab03b21661470f7320";
const SEGMENT_DIGEST: &str = "b7542dced2ab770894a14d1d04b066e3a899942602c5986d35ba6df6c1a35cfc";
const RETAINED_SEGMENT_LIMIT: u64 = 1_048_576;

#[test]
fn restart_reconstructs_the_exact_published_snapshot() -> Result<(), Box<dyn Error>> {
    let store = StoreFixture::create("restart-published")?;
    let loaded = FilesystemCatalogSnapshot::load(store.path(), restart_policy()?)?;
    let snapshot = loaded.snapshot()?;
    let identity = SegmentRecordIdentity::Chunk(ChunkId::hash_bytes(&[0])?);

    assert_eq!(loaded.generation().get(), 1);
    assert_eq!(
        snapshot
            .record(identity)
            .ok_or("restart snapshot omitted its record")?
            .payload(),
        [0]
    );
    store.remove()?;
    Ok(())
}

struct StoreFixture {
    sandbox: TestDirectory,
    catalog_path: PathBuf,
    segment_path: PathBuf,
}

impl StoreFixture {
    /// Initializes a store and publishes the one-zero chunk through the writer.
    ///
    /// The published HEAD, catalog, and segment must equal the frozen one-zero
    /// corpus bytes, so every law below reads exactly those artifacts.
    fn create(name: &str) -> Result<Self, Box<dyn Error>> {
        let sandbox = TestDirectory::create(name)?;
        publish_one_zero(sandbox.path())?;
        let catalog_path = sandbox
            .path()
            .join(format!("catalogs/0000000000000001-{CATALOG_DIGEST}.cat"));
        let segment_path = sandbox
            .path()
            .join(format!("segments/{SEGMENT_DIGEST}.seg"));
        for (path, hex) in [
            (sandbox.path().join("HEAD"), HEAD_HEX),
            (catalog_path.clone(), CATALOG_HEX),
            (segment_path.clone(), SEGMENT_HEX),
        ] {
            if fs::read(&path)? != fixture(hex)? {
                return Err(
                    format!("writer output {} differs from the corpus", path.display()).into(),
                );
            }
        }
        Ok(Self {
            sandbox,
            catalog_path,
            segment_path,
        })
    }

    fn path(&self) -> &Path {
        self.sandbox.path()
    }

    fn remove(self) -> Result<(), Box<dyn Error>> {
        self.sandbox.remove().map_err(Into::into)
    }
}

fn restart_policy() -> Result<CatalogRestartPolicy, Box<dyn Error>> {
    Ok(CatalogRestartPolicy::new(
        segment_policy(),
        CatalogRestartByteLimit::new(RETAINED_SEGMENT_LIMIT)?,
    ))
}

fn fixture(hex: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    decode_hex(hex.strip_suffix('\n').ok_or("fixture must end in one LF")?).map_err(Into::into)
}

/// Publishes generation one holding only the one-zero chunk record.
fn publish_one_zero(root: &Path) -> Result<(), Box<dyn Error>> {
    let admission = FilesystemVersionTwoAdmission::initialize(root)?.into_admission();
    let mut publisher = FilesystemVersionTwoCatalogPublisher::open(admission, restart_policy()?)?;
    let sealed = StagedSegment::begin(
        publisher.create_segment_stage()?,
        SegmentRecordLimit::MAXIMUM,
    )?
    .append(AdmittedSegmentRecord::for_chunk(&[0])?)?
    .seal()?;
    let bytes = fs::read(root.join("staging/current.seg"))?;
    let segments = [AdmittedSegment::decode(&bytes, segment_policy())?];
    let selection = publisher.select_segment(sealed, &segments[0])?;
    let catalog = CanonicalCatalog::from_segments(CatalogGeneration::new(1)?, None, &segments)?;
    let _receipt = publish_retained_catalog_generation(
        &mut publisher,
        CatalogPublicationExpectation::uninitialized(),
        selection,
        &catalog,
        &segments,
    )?;
    Ok(())
}

const fn segment_policy() -> SegmentReadPolicy {
    SegmentReadPolicy::new(SegmentRecordLimit::MAXIMUM, LayoutEntryLimit::MAXIMUM)
}
