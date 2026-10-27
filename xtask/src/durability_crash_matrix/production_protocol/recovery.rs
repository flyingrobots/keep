//! This module owns execution of the production recovery-discard protocol.

use std::fs;
use std::io::{self, Write};
use std::path::Path;

use keep::{
    AdmittedSegment, CanonicalPublicationHead, CatalogPublicationBoundary, CatalogPublicationError,
    CatalogPublicationExpectation, CatalogPublicationObserver, CatalogPublicationPhase,
    FilesystemRecoveryStageDiscarder, FilesystemVersionTwoRecoveryAdmission, RecoveryStage,
    RecoveryStageDiscardRequest, RecoveryStageMetadata, RetainedCatalogPublicationError,
    admit_recovery_stage_bytes, assess_recovery_stage, execute_recovery_stage_discard,
    fingerprint_recovery_stage, plan_recovery_stage_discard, publish_retained_catalog_generation,
};
use xtask::DurabilityCrashPoint;

use super::control::CrashControl;
use super::fixture::GoldenFixture;
use super::recovery_storage::CrashRecoveryStorage;
use super::{DurabilityCrashMatrixError, verification};
use super::{initialization, publication};

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
    let admission =
        FilesystemVersionTwoRecoveryAdmission::reopen_unchecked_for_repository_tasks(store_root)
            .map_err(|source| verification("admit production recovery store", source))?;
    let storage = FilesystemRecoveryStageDiscarder::open_version_two(admission)
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

/// Leaves the strict `head.next` prefix an interrupted first publication leaves.
///
/// The writer publishes the one-zero catalog through the gate; its observer
/// writes the prefix into the `head.next` the writer created and then fails
/// the head write, so the pools hold the published segment and catalog.
fn prepare_head(store_root: &Path) -> Result<RecoveryStage, DurabilityCrashMatrixError> {
    let mut publisher = initialization::publisher(store_root)?;
    let sealed = publication::stage_one_zero(&publisher)?;
    let segment_bytes = fs::read(store_root.join("staging/current.seg"))
        .map_err(|source| DurabilityCrashMatrixError::io("read recovery head segment", source))?;
    let segments = [
        AdmittedSegment::decode(&segment_bytes, initialization::segment_policy())
            .map_err(|source| verification("admit recovery head segment", source))?,
    ];
    let catalog = publication::first_catalog(&segments)?;
    let selection = publisher
        .select_segment(sealed, &segments[0])
        .map_err(|source| verification("select recovery head segment", source))?;
    let head = CanonicalPublicationHead::for_catalog(catalog.checksummed());
    let torn = head
        .encoded()
        .get(..HEAD_INTERRUPTION)
        .ok_or(DurabilityCrashMatrixError::FixtureRange)?
        .to_vec();
    publisher.observe_catalog_phases_for_repository_tasks(interrupt_head(store_root, torn));
    require_interrupted_head(publish_retained_catalog_generation(
        &mut publisher,
        CatalogPublicationExpectation::uninitialized(),
        selection,
        &catalog,
        &segments,
    ))
}

/// Requires the precondition publication to have failed exactly at the head write.
fn require_interrupted_head(
    interrupted: Result<keep::RetainedCatalogPublicationReceipt, RetainedCatalogPublicationError>,
) -> Result<RecoveryStage, DurabilityCrashMatrixError> {
    match interrupted {
        Err(RetainedCatalogPublicationError::Catalog {
            source:
                CatalogPublicationError::Storage {
                    phase: CatalogPublicationPhase::WriteHead,
                    ..
                },
        }) => Ok(RecoveryStage::NextHead),
        Err(source) => Err(verification("interrupt recovery head precondition", source)),
        Ok(_receipt) => Err(DurabilityCrashMatrixError::io(
            "interrupt recovery head precondition",
            io::Error::other("the precondition publication completed"),
        )),
    }
}

/// Writes `torn` into the created `head.next`, then fails the head write.
fn interrupt_head(store_root: &Path, torn: Vec<u8>) -> CatalogPublicationObserver {
    let stage = store_root.join("head.next");
    Box::new(move |phase, boundary| {
        if phase != CatalogPublicationPhase::WriteHead
            || boundary != CatalogPublicationBoundary::Before
        {
            return Ok(());
        }
        fs::OpenOptions::new()
            .write(true)
            .open(&stage)?
            .write_all(&torn)?;
        Err(io::Error::other(
            "recovery precondition interrupts the head write",
        ))
    })
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
