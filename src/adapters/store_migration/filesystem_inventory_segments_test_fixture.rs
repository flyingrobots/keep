//! Deterministic filesystem migration segment-pool fixture.

use std::error::Error;
use std::fs;
use std::path::Path;

use cap_std::ambient_authority;
use cap_std::fs::Dir;

use crate::LayoutEntryLimit;
use crate::adapters::filesystem_test_sandbox::TestDirectory;
use crate::adapters::test_support::decode_hex;
use crate::adapters::{
    AdmittedSegment, SegmentDigest, SegmentReadPolicy, SegmentRecordLimit, physical_pool_name,
};

const SEGMENT_HEX: &str =
    include_str!("../../../conformance/segment-store/v1/one-zero-segment.hex");
const EMPTY_SEGMENT_HEX: &str =
    include_str!("../../../conformance/segment-store/v1/empty-segment.hex");

pub(super) struct SegmentPoolFixture {
    sandbox: TestDirectory,
}

impl SegmentPoolFixture {
    pub(super) fn create(name: &str) -> Result<Self, Box<dyn Error>> {
        let sandbox = TestDirectory::create(name)?;
        fs::create_dir(sandbox.path().join("segments"))?;
        Ok(Self { sandbox })
    }

    pub(super) fn path(&self) -> &Path {
        self.sandbox.path()
    }

    pub(super) fn pool_path(&self) -> std::path::PathBuf {
        self.path().join("segments")
    }

    pub(super) fn open(&self) -> Result<Dir, Box<dyn Error>> {
        Ok(Dir::open_ambient_dir(
            self.pool_path(),
            ambient_authority(),
        )?)
    }

    pub(super) fn write_segment(&self, bytes: &[u8]) -> Result<SegmentDigest, Box<dyn Error>> {
        let segment = AdmittedSegment::decode(bytes, maximum_policy())?;
        fs::write(
            self.pool_path()
                .join(physical_pool_name::segment(segment.digest())),
            bytes,
        )?;
        Ok(segment.digest())
    }

    pub(super) fn write_named(&self, name: &str, bytes: &[u8]) -> Result<(), Box<dyn Error>> {
        fs::write(self.pool_path().join(name), bytes)?;
        Ok(())
    }

    pub(super) fn remove(self) -> Result<(), Box<dyn Error>> {
        self.sandbox.remove()?;
        Ok(())
    }
}

pub(super) fn one_zero_bytes() -> Result<Vec<u8>, Box<dyn Error>> {
    Ok(decode_hex(SEGMENT_HEX.trim())?)
}

pub(super) fn empty_bytes() -> Result<Vec<u8>, Box<dyn Error>> {
    Ok(decode_hex(EMPTY_SEGMENT_HEX.trim())?)
}

pub(super) const fn maximum_policy() -> SegmentReadPolicy {
    SegmentReadPolicy::new(SegmentRecordLimit::MAXIMUM, LayoutEntryLimit::MAXIMUM)
}
