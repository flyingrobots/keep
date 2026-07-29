//! Published catalog restart-loading laws.

#[path = "catalog_restart/refusal_laws.rs"]
mod refusal_laws;
#[path = "segment_filesystem_stage/sandbox.rs"]
pub mod sandbox;
mod support;

use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use keep::{
    CatalogRestartByteLimit, CatalogRestartPolicy, ChunkId, FilesystemCatalogSnapshot,
    LayoutEntryLimit, SegmentReadPolicy, SegmentRecordIdentity, SegmentRecordLimit,
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
    fn create(name: &str) -> Result<Self, Box<dyn Error>> {
        let sandbox = TestDirectory::create(name)?;
        let catalogs = sandbox.path().join("catalogs");
        let segments = sandbox.path().join("segments");
        fs::create_dir(&catalogs)?;
        fs::create_dir(&segments)?;
        fs::write(sandbox.path().join("HEAD"), fixture(HEAD_HEX)?)?;
        let catalog_path = catalogs.join(format!("0000000000000001-{CATALOG_DIGEST}.cat"));
        let segment_path = segments.join(format!("{SEGMENT_DIGEST}.seg"));
        fs::write(&catalog_path, fixture(CATALOG_HEX)?)?;
        fs::write(&segment_path, fixture(SEGMENT_HEX)?)?;
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
        SegmentReadPolicy::new(SegmentRecordLimit::MAXIMUM, LayoutEntryLimit::MAXIMUM),
        CatalogRestartByteLimit::new(RETAINED_SEGMENT_LIMIT)?,
    ))
}

fn fixture(hex: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    decode_hex(hex.strip_suffix('\n').ok_or("fixture must end in one LF")?).map_err(Into::into)
}
