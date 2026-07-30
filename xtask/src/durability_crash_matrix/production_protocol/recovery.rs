//! This module owns execution of the production recovery-discard protocol.

use std::fs;
use std::io::Write;
use std::path::Path;

use keep::{
    AdmittedSegment, CanonicalCatalog, CanonicalPublicationHead, CatalogGeneration,
    CatalogPublicationStorage, FilesystemRecoveryStageDiscarder, RecoveryStage,
    RecoveryStageDiscardRequest, RecoveryStageMetadata, admit_recovery_stage_bytes,
    assess_recovery_stage, execute_recovery_stage_discard, fingerprint_recovery_stage,
    plan_recovery_stage_discard,
};
use xtask::DurabilityCrashPoint;

use super::control::CrashControl;
use super::fixture::GoldenFixture;
use super::initialization;
use super::recovery_storage::CrashRecoveryStorage;
use super::{DurabilityCrashMatrixError, verification};

const SEGMENT_INTERRUPTION: usize = 32;
const HEAD_INTERRUPTION: usize = 64;

pub(super) fn run(
    store_root: &Path,
    control: &mut CrashControl,
) -> Result<(), DurabilityCrashMatrixError> {
    let stage = if targets_segment(control) {
        prepare_segment(store_root)?
    } else {
        prepare_head(store_root)?
    };
    let request = discard_request(store_root, stage)?;
    let storage = FilesystemRecoveryStageDiscarder::open_unchecked_for_repository_tasks(store_root)
        .map_err(|source| verification("open production recovery discarder", source))?;
    let mut storage = CrashRecoveryStorage::new(storage, control);
    execute_recovery_stage_discard(&mut storage, request)
        .map(|_receipt| ())
        .map_err(|source| verification("execute production recovery discard", source))
}

fn targets_segment(control: &CrashControl) -> bool {
    control
        .position(DurabilityCrashPoint::RemoveRecoveryStage)
        .is_some()
        || control
            .position(DurabilityCrashPoint::SynchronizeStagingAfterRecovery)
            .is_some()
}

fn prepare_segment(store_root: &Path) -> Result<RecoveryStage, DurabilityCrashMatrixError> {
    let publisher = initialization::publisher(store_root)?;
    let mut stage = publisher
        .create_segment_stage()
        .map_err(|source| verification("create recovery segment precondition", source))?;
    let fixture = GoldenFixture::segment()?;
    stage
        .write_all(fixture.prefix(SEGMENT_INTERRUPTION)?)
        .map_err(|source| {
            DurabilityCrashMatrixError::io("write recovery segment precondition", source)
        })?;
    drop(stage);
    drop(publisher);
    Ok(RecoveryStage::Segment)
}

fn prepare_head(store_root: &Path) -> Result<RecoveryStage, DurabilityCrashMatrixError> {
    let mut publisher = initialization::publisher(store_root)?;
    let segment_fixture = GoldenFixture::segment()?;
    let segment =
        AdmittedSegment::decode(segment_fixture.bytes(), initialization::segment_policy())
            .map_err(|source| verification("admit recovery head segment", source))?;
    let segments = [segment];
    let generation = CatalogGeneration::new(1)
        .map_err(|source| verification("construct recovery head generation", source))?;
    let catalog = CanonicalCatalog::from_segments(generation, None, &segments)
        .map_err(|source| verification("encode recovery head catalog", source))?;
    let head = CanonicalPublicationHead::for_catalog(catalog.checksummed());
    publisher.create_head_stage().map_err(|source| {
        DurabilityCrashMatrixError::io("create recovery head precondition", source)
    })?;
    publisher
        .write_head_prefix_for_repository_tasks(&head, HEAD_INTERRUPTION)
        .map_err(|source| {
            DurabilityCrashMatrixError::io("write recovery head precondition", source)
        })?;
    drop(publisher);
    Ok(RecoveryStage::NextHead)
}

fn discard_request(
    store_root: &Path,
    stage: RecoveryStage,
) -> Result<RecoveryStageDiscardRequest, DurabilityCrashMatrixError> {
    let path = match stage {
        RecoveryStage::Segment => store_root.join("staging/current.seg"),
        RecoveryStage::Catalog => store_root.join("staging/current.cat"),
        RecoveryStage::NextHead => store_root.join("head.next"),
    };
    let bytes = fs::read(path)
        .map_err(|source| DurabilityCrashMatrixError::io("read recovery precondition", source))?;
    let length = u64::try_from(bytes.len())
        .map_err(|source| verification("convert recovery precondition length", source))?;
    let metadata = RecoveryStageMetadata::new(stage, length)
        .map_err(|source| verification("admit recovery precondition metadata", source))?;
    let evidence = fingerprint_recovery_stage(metadata, bytes.as_slice())
        .map_err(|source| verification("fingerprint recovery precondition", source))?;
    let admitted = admit_recovery_stage_bytes(stage, evidence, &bytes)
        .map_err(|source| verification("admit recovery precondition bytes", source))?;
    let assessment = assess_recovery_stage(&admitted, initialization::segment_policy())
        .map_err(|source| verification("assess recovery precondition", source))?;
    plan_recovery_stage_discard(&assessment)
        .map_err(|source| verification("plan production recovery discard", source))
}
