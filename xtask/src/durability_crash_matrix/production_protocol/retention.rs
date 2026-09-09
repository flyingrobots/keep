//! This module owns execution of the production retention publication protocol.

use std::fs;
use std::path::Path;

use keep::{
    AdmittedCatalog, AdmittedRetentionRoot, AdmittedSegment, ChecksummedCatalog,
    ChecksummedPublicationHead, FilesystemRetentionPublicationAuthority,
    FilesystemStoreMigrationAuthority, FilesystemVersionTwoAdmission,
    RetentionGenerationExpectation, RetentionPublicationPreparation, execute_retention_publication,
    execute_store_migration, preflight_retention_transition, prepare_retention_publication,
};

use super::control::CrashControl;
use super::fixture::{BUNDLE_CATALOG_NAME, BUNDLE_SEGMENT_NAME, GoldenFixture};
use super::initialization;
use super::retention_storage::CrashRetentionStorage;
use super::{DurabilityCrashMatrixError, verification};

/// Migrates a fresh bundle store and publishes retention generation one,
/// dying at the selected coordinate.
pub(super) fn run(
    store_root: &Path,
    control: &mut CrashControl,
) -> Result<(), DurabilityCrashMatrixError> {
    let authority = migrated_authority(store_root)?;
    let root = GoldenFixture::retention_root()?;
    let preparation = preparation(root.bytes())?;
    let mut storage = CrashRetentionStorage::new(authority, control, store_root);
    execute_retention_publication(&mut storage, &preparation)
        .map(|_receipt| ())
        .map_err(|source| verification("execute production retention publication", source))
}

/// Initializes, populates, and migrates the bundle store, then reopens it as
/// version two and returns retention authority over it.
fn migrated_authority(
    store_root: &Path,
) -> Result<FilesystemRetentionPublicationAuthority, DurabilityCrashMatrixError> {
    let lock = initialization::initialized_lock(store_root)?;
    write_bundle(store_root)?;
    let mut migration = FilesystemStoreMigrationAuthority::open_unchecked_for_repository_tasks(
        lock,
        initialization::segment_policy(),
    )
    .map_err(|source| verification("open crash migration authority", source))?;
    let intent = migration
        .observe_intent()
        .map_err(|source| verification("observe crash migration intent", source))?;
    let _receipt = execute_store_migration(&mut migration, &intent)
        .map_err(|source| verification("execute crash store migration", source))?;
    drop(migration);
    reopened_authority(store_root)
}

/// Reopens the migrated store and returns retention authority over it.
pub(in crate::durability_crash_matrix) fn reopened_authority(
    store_root: &Path,
) -> Result<FilesystemRetentionPublicationAuthority, DurabilityCrashMatrixError> {
    let admission =
        FilesystemVersionTwoAdmission::reopen_unchecked_for_repository_tasks(store_root)
            .map_err(|source| verification("reopen crash store as version two", source))?;
    FilesystemRetentionPublicationAuthority::open(admission)
        .map_err(|source| verification("open crash retention authority", source))
}

fn write_bundle(store_root: &Path) -> Result<(), DurabilityCrashMatrixError> {
    let segment = GoldenFixture::bundle_segment()?;
    let catalog = GoldenFixture::bundle_catalog()?;
    let head = GoldenFixture::bundle_head()?;
    for (relative, bytes) in [
        (format!("segments/{BUNDLE_SEGMENT_NAME}"), segment.bytes()),
        (format!("catalogs/{BUNDLE_CATALOG_NAME}"), catalog.bytes()),
        ("HEAD".to_owned(), head.bytes()),
    ] {
        fs::write(store_root.join(&relative), bytes)
            .map_err(|source| DurabilityCrashMatrixError::io("write bundle corpus", source))?;
    }
    Ok(())
}

/// Prepares the frozen generation-one root as an initial publication against
/// the bundle catalog snapshot.
pub(in crate::durability_crash_matrix) fn preparation(
    root_bytes: &[u8],
) -> Result<RetentionPublicationPreparation<'_>, DurabilityCrashMatrixError> {
    let segment_fixture = GoldenFixture::bundle_segment()?;
    let catalog_fixture = GoldenFixture::bundle_catalog()?;
    let head_fixture = GoldenFixture::bundle_head()?;
    let candidate = AdmittedRetentionRoot::decode(root_bytes)
        .map_err(|source| verification("decode crash retention root", source))?;
    let segment =
        AdmittedSegment::decode(segment_fixture.bytes(), initialization::segment_policy())
            .map_err(|source| verification("admit bundle segment", source))?;
    let segments = [segment];
    let catalog: AdmittedCatalog<'_, '_> = ChecksummedCatalog::decode(catalog_fixture.bytes())
        .map_err(|source| verification("decode bundle catalog", source))?
        .admit(&segments)
        .map_err(|source| verification("admit bundle catalog", source))?;
    let head = ChecksummedPublicationHead::decode(head_fixture.bytes())
        .map_err(|source| verification("decode bundle head", source))?;
    let snapshot = head
        .admit(catalog)
        .map_err(|source| verification("admit bundle snapshot", source))?;
    let preflight = preflight_retention_transition(
        RetentionGenerationExpectation::Absent,
        None,
        candidate,
        &snapshot,
    )
    .map_err(|source| verification("preflight crash retention transition", source))?;
    prepare_retention_publication(preflight, None)
        .map_err(|source| verification("prepare crash retention publication", source))
}
