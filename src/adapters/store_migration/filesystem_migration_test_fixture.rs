//! This test module owns one exact published version-1 migration fixture.

use std::error::Error;
use std::fs;

use super::filesystem_migration_authority::FilesystemStoreMigrationAuthority;
use crate::adapters::FilesystemPlatformAdmission;
use crate::adapters::filesystem_test_sandbox::TestDirectory;
use crate::adapters::test_support::decode_hex;

const SEGMENT_HEX: &str =
    include_str!("../../../conformance/segment-store/v1/one-zero-segment.hex");
const CATALOG_HEX: &str =
    include_str!("../../../conformance/segment-store/v1/one-zero-catalog.hex");
const HEAD_HEX: &str = include_str!("../../../conformance/segment-store/v1/one-zero-head.hex");
const CATALOG_NAME: &str =
    "0000000000000001-04b82519b0399baefd0b9c0f32a871052e4c47e3a00226ab03b21661470f7320.cat";
const SEGMENT_NAME: &str = "b7542dced2ab770894a14d1d04b066e3a899942602c5986d35ba6df6c1a35cfc.seg";

pub(super) fn open_authority(
    name: &str,
) -> Result<(TestDirectory, FilesystemStoreMigrationAuthority), Box<dyn Error>> {
    let sandbox = TestDirectory::create(name)?;
    let admission = FilesystemPlatformAdmission::initialize_unchecked_for_tests(sandbox.path())?;
    fs::write(
        sandbox.path().join("segments").join(SEGMENT_NAME),
        decode_hex(SEGMENT_HEX.trim())?,
    )?;
    fs::write(
        sandbox.path().join("catalogs").join(CATALOG_NAME),
        decode_hex(CATALOG_HEX.trim())?,
    )?;
    fs::write(sandbox.path().join("HEAD"), decode_hex(HEAD_HEX.trim())?)?;
    let authority = FilesystemStoreMigrationAuthority::open(admission, maximum_policy())?;
    Ok((sandbox, authority))
}

pub(super) const fn maximum_policy() -> crate::adapters::SegmentReadPolicy {
    super::filesystem_inventory_catalogs_test_fixture::maximum_policy()
}
