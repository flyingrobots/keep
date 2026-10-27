//! This module owns crash-child store construction through the public writer.
//!
//! It is the crash matrix's counterpart of `tests/support/populated_store.rs`:
//! the same public entry points, under the matrix's typed error, because a
//! repository task cannot include integration-test support. Today creation is
//! version-one initialization, the first catalog through
//! `FilesystemCatalogPublisher`, and migration. When direct initialization
//! replaces migration (#198), only [`create_bundle_store`] changes.

use std::fs;
use std::path::Path;

use keep::{
    AdmittedSegment, CanonicalCatalog, CatalogGeneration, CatalogPublicationExpectation,
    FilesystemCatalogPublisher, FilesystemPlatformAdmission, FilesystemSegmentStage,
    FilesystemStoreMigrationAuthority, SealedSegment, SegmentRecordLimit, StagedSegment,
    execute_store_migration, publish_catalog_generation,
};

use super::crash_blob::CrashBlob;
use super::fixture::{BUNDLE_CATALOG_NAME, BUNDLE_SEGMENT_NAME, GoldenFixture};
use super::{DurabilityCrashMatrixError, initialization, verification};

/// Creates the one-zero bundle store at the empty directory `store_root`.
///
/// Writer authority is released before this returns. Crash rows prepare
/// against the frozen bundle snapshot and retention root, so the pools must
/// hold exactly the frozen bundle bytes; any other bytes refuse here.
pub(super) fn create_bundle_store(store_root: &Path) -> Result<(), DurabilityCrashMatrixError> {
    publish_bundle_catalog(store_root)?;
    migrate(store_root)?;
    require_frozen_bundle(store_root)
}

fn publish_bundle_catalog(store_root: &Path) -> Result<(), DurabilityCrashMatrixError> {
    let mut publisher = initialization::publisher(store_root)?;
    let blob = CrashBlob::bundle()?;
    let sealed = stage_bundle(&publisher, &blob)?;
    let bytes = fs::read(store_root.join("staging/current.seg"))
        .map_err(|source| DurabilityCrashMatrixError::io("read bundle segment", source))?;
    let segments = [
        AdmittedSegment::decode(&bytes, initialization::segment_policy())
            .map_err(|source| verification("admit bundle segment", source))?,
    ];
    let selection = publisher
        .select_segment(sealed, &segments[0])
        .map_err(|source| verification("select bundle segment stage", source))?;
    let generation = CatalogGeneration::new(1)
        .map_err(|source| verification("construct bundle catalog generation", source))?;
    let catalog = CanonicalCatalog::from_segments(generation, None, &segments)
        .map_err(|source| verification("encode bundle catalog", source))?;
    publish_catalog_generation(
        &mut publisher,
        CatalogPublicationExpectation::uninitialized(),
        selection,
        &catalog,
        &segments,
    )
    .map(|_receipt| ())
    .map_err(|source| verification("publish bundle catalog", source))
}

fn stage_bundle<'publisher>(
    publisher: &'publisher FilesystemCatalogPublisher,
    blob: &CrashBlob,
) -> Result<SealedSegment<FilesystemSegmentStage<'publisher>>, DurabilityCrashMatrixError> {
    let stage = publisher
        .create_segment_stage()
        .map_err(|source| verification("create bundle segment stage", source))?;
    let mut stage = StagedSegment::begin(stage, SegmentRecordLimit::MAXIMUM)
        .map_err(|source| verification("write bundle segment header", source))?;
    for record in blob.records()? {
        stage = stage
            .append(record)
            .map_err(|source| verification("append bundle segment record", source))?;
    }
    stage
        .seal()
        .map_err(|source| verification("seal bundle segment", source))
}

fn migrate(store_root: &Path) -> Result<(), DurabilityCrashMatrixError> {
    let admission = FilesystemPlatformAdmission::reopen(store_root)
        .map_err(|source| verification("reopen bundle store for migration", source))?;
    let mut migration =
        FilesystemStoreMigrationAuthority::open(admission, initialization::segment_policy())
            .map_err(|source| verification("open crash migration authority", source))?;
    let intent = migration
        .observe_intent()
        .map_err(|source| verification("observe crash migration intent", source))?;
    execute_store_migration(&mut migration, &intent)
        .map(|_receipt| ())
        .map_err(|source| verification("execute crash store migration", source))
}

fn require_frozen_bundle(store_root: &Path) -> Result<(), DurabilityCrashMatrixError> {
    for (artifact, relative, expected) in [
        (
            "bundle segment",
            format!("segments/{BUNDLE_SEGMENT_NAME}"),
            GoldenFixture::bundle_segment()?,
        ),
        (
            "bundle catalog",
            format!("catalogs/{BUNDLE_CATALOG_NAME}"),
            GoldenFixture::bundle_catalog()?,
        ),
        (
            "bundle head",
            "HEAD".to_owned(),
            GoldenFixture::bundle_head()?,
        ),
    ] {
        let observed = fs::read(store_root.join(relative))
            .map_err(|source| DurabilityCrashMatrixError::io("read writer-built bundle", source))?;
        if observed != expected.bytes() {
            return Err(DurabilityCrashMatrixError::artifact_bytes(
                artifact,
                expected.bytes(),
                &observed,
            ));
        }
    }
    Ok(())
}
