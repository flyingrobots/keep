//! Deterministic filesystem migration catalog-pool fixture.

use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use cap_std::ambient_authority;
use cap_std::fs::Dir;

use crate::LayoutEntryLimit;
use crate::adapters::filesystem_test_sandbox::TestDirectory;
use crate::adapters::test_support::decode_hex;
use crate::adapters::{
    AdmittedSegment, ChecksummedCatalog, SegmentReadPolicy, SegmentRecordLimit, physical_pool_name,
};

const SEGMENT_HEX: &str =
    include_str!("../../../conformance/segment-store/v1/one-zero-segment.hex");
const CATALOG_HEX: &str =
    include_str!("../../../conformance/segment-store/v1/one-zero-catalog.hex");
const EMPTY_SEGMENT_HEX: &str =
    include_str!("../../../conformance/segment-store/v1/empty-segment.hex");

pub(super) struct CatalogPoolFixture {
    sandbox: TestDirectory,
    segment_bytes: Vec<u8>,
    catalog_bytes: Vec<u8>,
}

impl CatalogPoolFixture {
    pub(super) fn create(name: &str) -> Result<Self, Box<dyn Error>> {
        let sandbox = TestDirectory::create(name)?;
        fs::create_dir(sandbox.path().join("segments"))?;
        fs::create_dir(sandbox.path().join("catalogs"))?;
        let segment_bytes = decode_hex(SEGMENT_HEX.trim())?;
        let catalog_bytes = decode_hex(CATALOG_HEX.trim())?;
        let segment = AdmittedSegment::decode(&segment_bytes, maximum_policy())?;
        let catalog = ChecksummedCatalog::decode(&catalog_bytes)?;
        fs::write(
            sandbox
                .path()
                .join("segments")
                .join(physical_pool_name::segment(segment.digest())),
            &segment_bytes,
        )?;
        fs::write(
            sandbox
                .path()
                .join("catalogs")
                .join(physical_pool_name::catalog(
                    catalog.generation(),
                    catalog.digest(),
                )),
            &catalog_bytes,
        )?;
        Ok(Self {
            sandbox,
            segment_bytes,
            catalog_bytes,
        })
    }

    pub(super) fn path(&self) -> &Path {
        self.sandbox.path()
    }

    pub(super) fn segments_path(&self) -> PathBuf {
        self.path().join("segments")
    }

    pub(super) fn catalogs_path(&self) -> PathBuf {
        self.path().join("catalogs")
    }

    pub(super) fn open_segments(&self) -> Result<Dir, Box<dyn Error>> {
        Ok(Dir::open_ambient_dir(
            self.segments_path(),
            ambient_authority(),
        )?)
    }

    pub(super) fn open_catalogs(&self) -> Result<Dir, Box<dyn Error>> {
        Ok(Dir::open_ambient_dir(
            self.catalogs_path(),
            ambient_authority(),
        )?)
    }

    pub(super) fn segment_bytes(&self) -> &[u8] {
        &self.segment_bytes
    }

    pub(super) fn catalog_bytes(&self) -> &[u8] {
        &self.catalog_bytes
    }

    pub(super) fn remove(self) -> Result<(), Box<dyn Error>> {
        self.sandbox.remove()?;
        Ok(())
    }
}

pub(super) const fn maximum_policy() -> SegmentReadPolicy {
    SegmentReadPolicy::new(SegmentRecordLimit::MAXIMUM, LayoutEntryLimit::MAXIMUM)
}

pub(super) fn empty_segment_bytes() -> Result<Vec<u8>, Box<dyn Error>> {
    Ok(decode_hex(EMPTY_SEGMENT_HEX.trim())?)
}
