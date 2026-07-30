//! This module owns execution of the production publication protocols.

use std::fs;
use std::path::Path;

use keep::{
    AdmittedSegment, AdmittedSegmentRecord, CanonicalCatalog, CatalogGeneration,
    CatalogPublicationExpectation, SegmentRecordLimit, StagedSegment, publish_catalog_generation,
};
use xtask::DurabilityCrashPoint;

use super::control::{CrashControl, DuringTiming};
use super::initialization;
use super::publication_storage::CrashPublicationStorage;
use super::segment_stage::CrashSegmentStage;
use super::{DurabilityCrashMatrixError, verification};

pub(super) fn run(
    store_root: &Path,
    control: &mut CrashControl,
) -> Result<(), DurabilityCrashMatrixError> {
    let publisher = initialization::publisher(store_root)?;
    let point = DurabilityCrashPoint::CreateSegmentStage;
    control
        .before(point, DuringTiming::After)
        .map_err(crash_gate)?;
    let stage = publisher
        .create_segment_stage()
        .map_err(|source| verification("create production segment stage", source))?;
    control
        .after(point, DuringTiming::After)
        .map_err(crash_gate)?;

    let stage = CrashSegmentStage::new(stage, control);
    let record = AdmittedSegmentRecord::for_chunk(&[0])
        .map_err(|source| verification("admit crash segment record", source))?;
    let sealed = StagedSegment::begin(stage, SegmentRecordLimit::MAXIMUM)
        .map_err(|source| verification("write production segment header", source))?
        .append(record)
        .map_err(|source| verification("append production segment record", source))?
        .seal()
        .map_err(|source| verification("seal production segment", source))?
        .map_stage(CrashSegmentStage::into_inner);

    let segment_bytes = fs::read(store_root.join("staging/current.seg"))
        .map_err(|source| DurabilityCrashMatrixError::io("read sealed crash segment", source))?;
    let segment = AdmittedSegment::decode(&segment_bytes, initialization::segment_policy())
        .map_err(|source| verification("admit sealed crash segment", source))?;
    let segments = [segment];
    let generation = CatalogGeneration::new(1)
        .map_err(|source| verification("construct crash catalog generation", source))?;
    let catalog = CanonicalCatalog::from_segments(generation, None, &segments)
        .map_err(|source| verification("encode crash catalog", source))?;
    let selection = publisher
        .select_segment(sealed, &segments[0])
        .map_err(|source| verification("select production segment stage", source))?;
    let mut storage = CrashPublicationStorage::new(publisher, control);
    publish_catalog_generation(
        &mut storage,
        CatalogPublicationExpectation::uninitialized(),
        selection,
        &catalog,
        &segments,
    )
    .map(|_receipt| ())
    .map_err(|source| verification("execute production catalog publication", source))
}

const fn crash_gate(source: std::io::Error) -> DurabilityCrashMatrixError {
    DurabilityCrashMatrixError::io("await production crash boundary", source)
}
