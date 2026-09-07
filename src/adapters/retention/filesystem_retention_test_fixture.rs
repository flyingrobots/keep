//! This test module owns one migrated version-2 retention publication fixture.

use std::error::Error;
use std::fs;

use super::filesystem_retention_authority::FilesystemRetentionPublicationAuthority;
use crate::LayoutEntryLimit;
use crate::adapters::filesystem_test_sandbox::TestDirectory;
use crate::adapters::test_support::decode_hex;
use crate::adapters::{
    AdmittedCatalog, AdmittedSegment, CatalogSnapshot, ChecksummedCatalog,
    ChecksummedPublicationHead, FilesystemPlatformAdmission, FilesystemStoreMigrationAuthority,
    SegmentReadPolicy, SegmentRecordLimit,
};
use crate::execute_store_migration;

/// Frozen canonical generation-one root.
pub(super) const ROOT_HEX: &str =
    include_str!("../../../conformance/segment-store/v2/one-anchor-root.hex");
/// Frozen canonical generation-one manifest.
pub(super) const MANIFEST_HEX: &str =
    include_str!("../../../conformance/segment-store/v2/one-root-manifest.hex");
/// Frozen canonical generation-one retention head.
pub(super) const HEAD_HEX: &str =
    include_str!("../../../conformance/segment-store/v2/one-root-head.hex");

const SEGMENT_HEX: &str =
    include_str!("../../../conformance/segment-store/v1/one-zero-bundle-segment.hex");
const CATALOG_HEX: &str =
    include_str!("../../../conformance/segment-store/v1/one-zero-bundle-catalog.hex");
const CATALOG_HEAD_HEX: &str =
    include_str!("../../../conformance/segment-store/v1/one-zero-bundle-head.hex");

const SEGMENT_NAME: &str = "221f6745cd8a5221c9a87c3707593608479282b54a4a74d0e753fd76f70e8db2.seg";
const CATALOG_NAME: &str =
    "0000000000000001-0b7cad1b6de663d34beacbc214db7497f2e36ab6b08dfbd5febbc8d06a418811.cat";

/// Builds one migrated version-2 store and pins its retention authority.
///
/// The fixture publishes the exact bundle version-1 corpus, executes the
/// complete forward migration, releases writer authority, then reopens the
/// admitted root for retention publication.
pub(super) fn open_authority(
    name: &str,
) -> Result<(TestDirectory, FilesystemRetentionPublicationAuthority), Box<dyn Error>> {
    let sandbox = migrated_store(name)?;
    let admission =
        FilesystemPlatformAdmission::reopen_version_two_unchecked_for_tests(sandbox.path())?;
    let authority = FilesystemRetentionPublicationAuthority::open(admission)?;
    Ok((sandbox, authority))
}

/// Decodes one LF-terminated lowercase hexadecimal conformance fixture.
pub(super) fn fixture(hex: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    decode_hex(hex.strip_suffix('\n').ok_or("fixture must end in one LF")?).map_err(Into::into)
}

/// Runs one operation against the frozen bundle catalog snapshot.
pub(super) fn with_snapshot<T>(
    operation: impl FnOnce(&CatalogSnapshot<'_, '_, '_>) -> T,
) -> Result<T, Box<dyn Error>> {
    let segment_bytes = fixture(SEGMENT_HEX)?;
    let catalog_bytes = fixture(CATALOG_HEX)?;
    let head_bytes = fixture(CATALOG_HEAD_HEX)?;
    let segment = AdmittedSegment::decode(&segment_bytes, maximum_policy())?;
    let segments = [segment];
    let catalog: AdmittedCatalog<'_, '_> =
        ChecksummedCatalog::decode(&catalog_bytes)?.admit(&segments)?;
    let head = ChecksummedPublicationHead::decode(&head_bytes)?;
    let snapshot = head.admit(catalog)?;
    Ok(operation(&snapshot))
}

fn migrated_store(name: &str) -> Result<TestDirectory, Box<dyn Error>> {
    let sandbox = TestDirectory::create(name)?;
    let admission = FilesystemPlatformAdmission::initialize_unchecked_for_tests(sandbox.path())?;
    write_version_one(&sandbox)?;
    let mut authority = FilesystemStoreMigrationAuthority::open(admission, maximum_policy())?;
    let intent = authority.observe_intent()?;
    let _receipt = execute_store_migration(&mut authority, &intent)?;
    drop(authority);
    Ok(sandbox)
}

fn write_version_one(sandbox: &TestDirectory) -> Result<(), Box<dyn Error>> {
    fs::write(
        sandbox.path().join("segments").join(SEGMENT_NAME),
        fixture(SEGMENT_HEX)?,
    )?;
    fs::write(
        sandbox.path().join("catalogs").join(CATALOG_NAME),
        fixture(CATALOG_HEX)?,
    )?;
    fs::write(sandbox.path().join("HEAD"), fixture(CATALOG_HEAD_HEX)?)?;
    Ok(())
}

const fn maximum_policy() -> SegmentReadPolicy {
    SegmentReadPolicy::new(SegmentRecordLimit::MAXIMUM, LayoutEntryLimit::MAXIMUM)
}
