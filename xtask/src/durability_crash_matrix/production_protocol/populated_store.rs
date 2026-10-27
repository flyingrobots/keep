//! This module owns crash-child store construction through the public writer.
//!
//! It is the crash matrix's counterpart of `tests/support/populated_store.rs`:
//! the same public entry points, under the matrix's typed error, because a
//! repository task cannot include integration-test support. Creation is direct
//! initialization, then the first catalog through
//! `FilesystemCatalogPublisher` and the retained-closure gate.

use std::fs;
use std::path::Path;

use keep::{
    AdmittedSegment, CanonicalCatalog, CatalogGeneration, CatalogPublicationExpectation,
    FilesystemCatalogPublisher, FilesystemSegmentStage, FilesystemStoreAdmission, SealedSegment,
    SegmentRecordLimit, StagedSegment, publish_retained_catalog_generation,
};

use super::crash_blob::CrashBlob;
use super::fixture::{BUNDLE_CATALOG_NAME, BUNDLE_SEGMENT_NAME, GoldenFixture};
use super::{DurabilityCrashMatrixError, retained_catalog, verification, writer};

/// Creates the one-zero bundle store at the empty directory `store_root`.
///
/// Writer authority is released before this returns. Crash rows prepare
/// against the frozen bundle snapshot and retention root, so the pools must
/// hold exactly the frozen bundle bytes; any other bytes refuse here.
pub(super) fn create_bundle_store(store_root: &Path) -> Result<(), DurabilityCrashMatrixError> {
    let admission = FilesystemStoreAdmission::initialize_unchecked_for_repository_tasks(store_root)
        .map_err(|source| verification("initialize crash bundle store", source))?
        .into_admission();
    publish_bundle_catalog(store_root, admission)?;
    require_frozen_bundle(store_root)
}

/// Publishes the bundle blob as the first catalog of an initialized store.
pub(in crate::durability_crash_matrix) fn publish_bundle_catalog(
    store_root: &Path,
    admission: FilesystemStoreAdmission,
) -> Result<(), DurabilityCrashMatrixError> {
    let mut publisher = FilesystemCatalogPublisher::open(admission, retained_catalog::policy()?)
        .map_err(|source| verification("open crash bundle publisher", source))?;
    let blob = CrashBlob::bundle()?;
    let sealed = stage_bundle(&publisher, &blob)?;
    let bytes = fs::read(store_root.join("staging/current.seg"))
        .map_err(|source| DurabilityCrashMatrixError::io("read bundle segment", source))?;
    let segments = [AdmittedSegment::decode(&bytes, writer::segment_policy())
        .map_err(|source| verification("admit bundle segment", source))?];
    let selection = publisher
        .select_segment(sealed, &segments[0])
        .map_err(|source| verification("select bundle segment stage", source))?;
    let generation = CatalogGeneration::new(1)
        .map_err(|source| verification("construct bundle catalog generation", source))?;
    let catalog = CanonicalCatalog::from_segments(generation, None, &segments)
        .map_err(|source| verification("encode bundle catalog", source))?;
    publish_retained_catalog_generation(
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

pub(in crate::durability_crash_matrix) fn require_frozen_bundle(
    store_root: &Path,
) -> Result<(), DurabilityCrashMatrixError> {
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
