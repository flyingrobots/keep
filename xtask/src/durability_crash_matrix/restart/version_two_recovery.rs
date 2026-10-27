//! This module owns version-two stage recovery after one writer process death.
//!
//! Each step reopens the store through the production version-two recovery
//! admission (without the platform probe), fingerprints the fixed stage under
//! the pinned inventory, admits exactly those bytes, and executes one planned
//! transition with the public recovery executors.

use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use keep::{
    CatalogPublicationExpectation, ChecksummedCatalog, ChecksummedPublicationHead,
    FilesystemCatalogSnapshot, FilesystemRecoveryInventoryReader,
    FilesystemRecoveryNextHeadFinalizer, FilesystemRecoveryStageCompleter,
    FilesystemRecoveryStageDiscarder, FilesystemVersionTwoRecoveryAdmission, RecoveryEntryRole,
    RecoveryInventoryLimit, RecoveryStage, admit_recovery_stage_bytes, assess_recovery_stage,
    classify_version_two_recovery_names, execute_recovery_next_head_finalization,
    execute_recovery_stage_completion, execute_recovery_stage_discard,
    plan_recovery_next_head_finalization, plan_recovery_stage_completion,
    plan_recovery_stage_discard,
};

use super::super::DurabilityCrashMatrixError;
use super::super::production_protocol::initialization::segment_policy;
use super::super::production_protocol::verification;
use super::super::production_protocol::version_two_catalog::{admit_segments, policy};

/// Every fixed catalog-protocol stage a version-two inventory classifies.
pub(super) fn stage_roles(
    root: &Path,
) -> Result<Vec<RecoveryEntryRole>, DurabilityCrashMatrixError> {
    let mut reader =
        FilesystemRecoveryInventoryReader::open_version_two_unchecked_for_repository_tasks(root)
            .map_err(|source| verification("open version-two recovery inventory", source))?;
    let limit = RecoveryInventoryLimit::new(4096)
        .map_err(|source| verification("bound version-two recovery inventory", source))?;
    let inventory = reader
        .read(limit)
        .map_err(|source| verification("read version-two recovery inventory", source))?;
    let manifest = classify_version_two_recovery_names(inventory)
        .map_err(|source| verification("classify version-two recovery names", source))?;
    Ok(manifest
        .entries()
        .iter()
        .map(keep::RecoveryNamedEntry::role)
        .filter(|role| {
            matches!(
                role,
                RecoveryEntryRole::SegmentStage
                    | RecoveryEntryRole::CatalogStage
                    | RecoveryEntryRole::NextHeadStage
            )
        })
        .collect())
}

/// Links the complete fixed segment stage into the segment pool.
pub(super) fn complete_segment(root: &Path) -> Result<(), DurabilityCrashMatrixError> {
    let bytes = stage_bytes(root, RecoveryStage::Segment)?;
    let evidence = evidence(root, RecoveryStage::Segment)?;
    let admitted = admit_recovery_stage_bytes(RecoveryStage::Segment, evidence, &bytes)
        .map_err(|source| verification("admit recovered segment stage", source))?;
    let assessment = assess_recovery_stage(&admitted, segment_policy())
        .map_err(|source| verification("assess recovered segment stage", source))?;
    let request = plan_recovery_stage_completion(&assessment)
        .map_err(|source| verification("plan recovered segment completion", source))?;
    let mut completer = FilesystemRecoveryStageCompleter::open_version_two(admission(root)?)
        .map_err(|source| verification("open version-two stage completer", source))?;
    execute_recovery_stage_completion(&mut completer, request)
        .map(|_receipt| ())
        .map_err(|source| verification("complete recovered segment stage", source))
}

/// Removes the incomplete fixed catalog stage.
pub(super) fn discard_catalog(root: &Path) -> Result<(), DurabilityCrashMatrixError> {
    let bytes = stage_bytes(root, RecoveryStage::Catalog)?;
    let evidence = evidence(root, RecoveryStage::Catalog)?;
    let admitted = admit_recovery_stage_bytes(RecoveryStage::Catalog, evidence, &bytes)
        .map_err(|source| verification("admit recovered catalog stage", source))?;
    let assessment = assess_recovery_stage(&admitted, segment_policy())
        .map_err(|source| verification("assess recovered catalog stage", source))?;
    let request = plan_recovery_stage_discard(&assessment)
        .map_err(|source| verification("plan recovered catalog discard", source))?;
    let mut discarder = FilesystemRecoveryStageDiscarder::open_version_two(admission(root)?)
        .map_err(|source| verification("open version-two stage discarder", source))?;
    execute_recovery_stage_discard(&mut discarder, request)
        .map(|_receipt| ())
        .map_err(|source| verification("discard recovered catalog stage", source))
}

/// Finalizes the complete `head.next` through the retained-closure gate.
pub(super) fn finalize_head(
    root: &Path,
    candidate_segments: &[&[u8]],
) -> Result<(), DurabilityCrashMatrixError> {
    let head_bytes = stage_bytes(root, RecoveryStage::NextHead)?;
    let evidence = evidence(root, RecoveryStage::NextHead)?;
    let head = ChecksummedPublicationHead::decode(&head_bytes)
        .map_err(|source| verification("decode recovered next head", source))?;
    let catalog_bytes =
        fs::read(root.join("catalogs").join(catalog_name(&head)?)).map_err(|source| {
            DurabilityCrashMatrixError::io("read recovered candidate catalog", source)
        })?;
    let segments = admit_segments(candidate_segments)?;
    let catalog = ChecksummedCatalog::decode(&catalog_bytes)
        .map_err(|source| verification("decode recovered candidate catalog", source))?
        .admit(&segments)
        .map_err(|source| verification("admit recovered candidate catalog", source))?;
    let candidate = head
        .admit(catalog)
        .map_err(|source| verification("admit recovered candidate snapshot", source))?;
    let admitted = admit_recovery_stage_bytes(RecoveryStage::NextHead, evidence, &head_bytes)
        .map_err(|source| verification("admit recovered next-head stage", source))?;
    let assessment = assess_recovery_stage(&admitted, segment_policy())
        .map_err(|source| verification("assess recovered next-head stage", source))?;
    let current = FilesystemCatalogSnapshot::load(root, policy()?)
        .map_err(|source| verification("load recovered current catalog", source))?;
    let expectation = CatalogPublicationExpectation::successor_of(
        &current
            .snapshot()
            .map_err(|source| verification("admit recovered current catalog", source))?,
    );
    let request = plan_recovery_next_head_finalization(&assessment, &candidate, expectation)
        .map_err(|source| verification("plan recovered next-head finalization", source))?;
    let mut finalizer =
        FilesystemRecoveryNextHeadFinalizer::open_version_two(admission(root)?, policy()?)
            .map_err(|source| verification("open version-two head finalizer", source))?;
    execute_recovery_next_head_finalization(&mut finalizer, request)
        .map(|_receipt| ())
        .map_err(|source| verification("finalize recovered next head", source))
}

fn admission(
    root: &Path,
) -> Result<FilesystemVersionTwoRecoveryAdmission, DurabilityCrashMatrixError> {
    FilesystemVersionTwoRecoveryAdmission::reopen_unchecked_for_repository_tasks(root)
        .map_err(|source| verification("admit version-two stage recovery", source))
}

fn evidence(
    root: &Path,
    stage: RecoveryStage,
) -> Result<keep::RecoveryStageEvidence, DurabilityCrashMatrixError> {
    FilesystemRecoveryInventoryReader::open_version_two_unchecked_for_repository_tasks(root)
        .map_err(|source| verification("open version-two stage inventory", source))?
        .fingerprint_stage(stage)
        .map_err(|source| verification("fingerprint version-two recovery stage", source))
}

fn stage_bytes(root: &Path, stage: RecoveryStage) -> Result<Vec<u8>, DurabilityCrashMatrixError> {
    fs::read(stage_path(root, stage))
        .map_err(|source| DurabilityCrashMatrixError::io("read version-two recovery stage", source))
}

fn stage_path(root: &Path, stage: RecoveryStage) -> PathBuf {
    match stage {
        RecoveryStage::Segment => root.join("staging/current.seg"),
        RecoveryStage::Catalog => root.join("staging/current.cat"),
        RecoveryStage::NextHead => root.join("head.next"),
    }
}

fn catalog_name(
    head: &ChecksummedPublicationHead<'_>,
) -> Result<String, DurabilityCrashMatrixError> {
    let mut name = format!("{:016x}-", head.generation().get());
    for byte in head.catalog_digest().as_bytes() {
        write!(name, "{byte:02x}")
            .map_err(|source| verification("name recovered candidate catalog", source))?;
    }
    name.push_str(".cat");
    Ok(name)
}
