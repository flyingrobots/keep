//! This module owns execution of the production version-two catalog writer.
//!
//! The child migrates the bundle store, publishes retention generation one,
//! then ingests one new blob through the version-two writer. An observer
//! installed on the writer's own stage type blocks at the selected boundary
//! for process death; no transition is reimplemented here.

use std::fs;
use std::path::Path;

use keep::{
    AdmittedSegment, CanonicalCatalog, CatalogGeneration, CatalogPublicationBoundary,
    CatalogPublicationExpectation, CatalogPublicationObserver, CatalogPublicationPhase,
    CatalogSnapshot, FilesystemVersionTwoAdmission, FilesystemVersionTwoCatalogPublisher,
    SegmentRecordLimit, StagedSegment, execute_retention_publication,
    publish_retained_catalog_generation,
};
use xtask::DurabilityCrashPoint;

use super::control::{CrashControl, DuringTiming};
use super::fixture::GoldenFixture;
use super::version_two_blob::VersionTwoBlob;
use super::{DurabilityCrashMatrixError, initialization, retention, verification};

/// Ingests the version-two crash blob, dying at the selected coordinate.
pub(super) fn run(
    store_root: &Path,
    control: CrashControl,
) -> Result<(), DurabilityCrashMatrixError> {
    retained_bundle_store(store_root)?;
    let admission =
        FilesystemVersionTwoAdmission::reopen_unchecked_for_repository_tasks(store_root)
            .map_err(|source| verification("reopen crash store for version-two writer", source))?;
    let mut publisher = FilesystemVersionTwoCatalogPublisher::open(admission, policy()?)
        .map_err(|source| verification("open version-two catalog publisher", source))?;
    publisher.observe_catalog_phases_for_repository_tasks(observer(control));
    let blob = VersionTwoBlob::identify()?;
    let mut stage = StagedSegment::begin(
        publisher
            .create_segment_stage()
            .map_err(|source| verification("create version-two segment stage", source))?,
        SegmentRecordLimit::MAXIMUM,
    )
    .map_err(|source| verification("write version-two segment header", source))?;
    for record in blob.records()? {
        stage = stage
            .append(record)
            .map_err(|source| verification("append version-two segment record", source))?;
    }
    let sealed = stage
        .seal()
        .map_err(|source| verification("seal version-two segment", source))?;
    let bundle = GoldenFixture::bundle_segment()?;
    let staged = fs::read(store_root.join("staging/current.seg"))
        .map_err(|source| DurabilityCrashMatrixError::io("read version-two segment", source))?;
    let segments = admit_segments(&[bundle.bytes(), &staged])?;
    let new_segment = segments
        .last()
        .ok_or(DurabilityCrashMatrixError::FixtureRange)?;
    let selection = publisher
        .select_segment(sealed, new_segment)
        .map_err(|source| verification("select version-two segment stage", source))?;
    let head = GoldenFixture::bundle_head()?;
    let catalog_fixture = GoldenFixture::bundle_catalog()?;
    let predecessor = segments
        .get(..1)
        .ok_or(DurabilityCrashMatrixError::FixtureRange)?;
    let current = bundle_snapshot(head.bytes(), catalog_fixture.bytes(), predecessor)?;
    let expectation = CatalogPublicationExpectation::successor_of(&current);
    let generation = CatalogGeneration::new(2)
        .map_err(|source| verification("construct version-two catalog generation", source))?;
    let catalog =
        CanonicalCatalog::from_segments(generation, Some(current.catalog_digest()), &segments)
            .map_err(|source| verification("encode version-two catalog", source))?;
    publish_retained_catalog_generation(&mut publisher, expectation, selection, &catalog, &segments)
        .map(|_receipt| ())
        .map_err(|source| verification("execute version-two catalog publication", source))
}

/// Migrates the bundle store and publishes its golden retention root.
pub(in crate::durability_crash_matrix) fn retained_bundle_store(
    store_root: &Path,
) -> Result<(), DurabilityCrashMatrixError> {
    let mut authority = retention::migrated_authority(store_root)?;
    let root = GoldenFixture::retention_root()?;
    let preparation = retention::preparation(root.bytes())?;
    execute_retention_publication(&mut authority, &preparation)
        .map(|_receipt| ())
        .map_err(|source| verification("publish crash retention generation one", source))
}

pub(in crate::durability_crash_matrix) fn policy()
-> Result<keep::CatalogRestartPolicy, DurabilityCrashMatrixError> {
    let limit = keep::CatalogRestartByteLimit::new(1_048_576)
        .map_err(|source| verification("bound version-two catalog read", source))?;
    Ok(keep::CatalogRestartPolicy::new(
        initialization::segment_policy(),
        limit,
    ))
}

pub(in crate::durability_crash_matrix) fn admit_segments<'bytes>(
    encoded: &[&'bytes [u8]],
) -> Result<Vec<AdmittedSegment<'bytes>>, DurabilityCrashMatrixError> {
    encoded
        .iter()
        .map(|bytes| {
            AdmittedSegment::decode(bytes, initialization::segment_policy())
                .map_err(|source| verification("admit version-two crash segment", source))
        })
        .collect()
}

fn bundle_snapshot<'bytes>(
    head: &'bytes [u8],
    catalog: &'bytes [u8],
    segments: &'bytes [AdmittedSegment<'bytes>],
) -> Result<CatalogSnapshot<'bytes, 'bytes, 'bytes>, DurabilityCrashMatrixError> {
    let catalog = keep::ChecksummedCatalog::decode(catalog)
        .map_err(|source| verification("decode bundle catalog", source))?
        .admit(segments)
        .map_err(|source| verification("admit bundle catalog", source))?;
    keep::ChecksummedPublicationHead::decode(head)
        .map_err(|source| verification("decode bundle head", source))?
        .admit(catalog)
        .map_err(|source| verification("admit bundle snapshot", source))
}

/// Maps the writer's own transitions onto the four version-two crash points.
fn observer(mut control: CrashControl) -> CatalogPublicationObserver {
    Box::new(move |phase, boundary| {
        let Some((point, timing)) = crash_point(phase) else {
            return Ok(());
        };
        match boundary {
            CatalogPublicationBoundary::Before => control.before(point, timing),
            CatalogPublicationBoundary::After => control.after(point, timing),
        }
    })
}

const fn crash_point(
    phase: CatalogPublicationPhase,
) -> Option<(DurabilityCrashPoint, DuringTiming)> {
    match phase {
        CatalogPublicationPhase::LinkSegment => Some((
            DurabilityCrashPoint::VersionTwoLinkSegment,
            DuringTiming::After,
        )),
        CatalogPublicationPhase::CreateCatalogStage => Some((
            DurabilityCrashPoint::VersionTwoCreateCatalogStage,
            DuringTiming::After,
        )),
        CatalogPublicationPhase::SynchronizeHead => Some((
            DurabilityCrashPoint::VersionTwoSynchronizeHead,
            DuringTiming::Before,
        )),
        CatalogPublicationPhase::ReplaceHead => Some((
            DurabilityCrashPoint::VersionTwoReplaceHead,
            DuringTiming::After,
        )),
        _ => None,
    }
}
