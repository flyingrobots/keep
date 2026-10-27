//! This module owns stage recovery through the public recovery tools.
//!
//! Every helper reopens the store through stage recovery admission,
//! fingerprints the fixed stage under the pinned inventory, admits exactly the
//! fingerprinted bytes, and executes one planned transition.

use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use keep::{
    CatalogPublicationExpectation, ChecksummedCatalog, ChecksummedPublicationHead,
    FilesystemCatalogSnapshot, FilesystemRecoveryAdmission, FilesystemRecoveryInventoryReader,
    FilesystemRecoveryNextHeadFinalizer, FilesystemRecoveryStageCompleter,
    FilesystemRecoveryStageDiscarder, RecoveryEntryRole, RecoveryInventoryLimit,
    RecoveryNextHeadFinalizationError, RecoveryNextHeadFinalizationReceipt, RecoveryStage,
    RecoveryStageCompletionReceipt, RecoveryStageDiscardReceipt, admit_recovery_stage_bytes,
    assess_recovery_stage, classify_recovery_names, execute_recovery_next_head_finalization,
    execute_recovery_stage_completion, execute_recovery_stage_discard,
    plan_recovery_next_head_finalization, plan_recovery_stage_completion,
    plan_recovery_stage_discard,
};

use super::durable_fixture;
use super::writer_fixture::{TestResult, admit_all, segment_policy};

/// Classifies a fresh inventory and returns every stage role.
pub(super) fn stage_roles(root: &Path) -> TestResult<Vec<RecoveryEntryRole>> {
    let mut reader = FilesystemRecoveryInventoryReader::open(root)?;
    let inventory = reader.read(RecoveryInventoryLimit::new(4096)?)?;
    let manifest = classify_recovery_names(inventory)?;
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

/// Links one complete segment or catalog stage into its immutable pool.
pub(super) fn complete(
    root: &Path,
    stage: RecoveryStage,
) -> TestResult<RecoveryStageCompletionReceipt> {
    let reader = FilesystemRecoveryInventoryReader::open(root)?;
    let evidence = reader.fingerprint_stage(stage)?;
    let bytes = fs::read(stage_path(root, stage))?;
    let admitted = admit_recovery_stage_bytes(stage, evidence, &bytes)?;
    let assessment = assess_recovery_stage(&admitted, segment_policy())?;
    let request = plan_recovery_stage_completion(&assessment)?;
    let admission = FilesystemRecoveryAdmission::reopen(root)?;
    let mut completer = FilesystemRecoveryStageCompleter::open(admission)?;
    Ok(execute_recovery_stage_completion(&mut completer, request)?)
}

/// Removes one exactly fingerprinted incomplete stage.
pub(super) fn discard(
    root: &Path,
    stage: RecoveryStage,
) -> TestResult<RecoveryStageDiscardReceipt> {
    let reader = FilesystemRecoveryInventoryReader::open(root)?;
    let evidence = reader.fingerprint_stage(stage)?;
    let bytes = fs::read(stage_path(root, stage))?;
    let admitted = admit_recovery_stage_bytes(stage, evidence, &bytes)?;
    let assessment = assess_recovery_stage(&admitted, segment_policy())?;
    let request = plan_recovery_stage_discard(&assessment)?;
    let admission = FilesystemRecoveryAdmission::reopen(root)?;
    let mut discarder = FilesystemRecoveryStageDiscarder::open(admission)?;
    Ok(execute_recovery_stage_discard(&mut discarder, request)?)
}

/// Plans and executes gated finalization of the complete `head.next`.
///
/// `candidate_segments` are the exact segments the staged catalog references.
/// The outer result carries fixture failures; the inner result is the
/// finalization outcome.
pub(super) fn finalize(
    root: &Path,
    candidate_segments: &[Vec<u8>],
) -> TestResult<Result<RecoveryNextHeadFinalizationReceipt, RecoveryNextHeadFinalizationError>> {
    let reader = FilesystemRecoveryInventoryReader::open(root)?;
    let evidence = reader.fingerprint_stage(RecoveryStage::NextHead)?;
    drop(reader);
    let head_bytes = fs::read(root.join("head.next"))?;
    let head = ChecksummedPublicationHead::decode(&head_bytes)?;
    let catalog_bytes = fs::read(root.join("catalogs").join(catalog_name(&head)?))?;
    let segments = admit_all(candidate_segments)?;
    let candidate = head.admit(ChecksummedCatalog::decode(&catalog_bytes)?.admit(&segments)?)?;
    let admitted = admit_recovery_stage_bytes(RecoveryStage::NextHead, evidence, &head_bytes)?;
    let assessment = assess_recovery_stage(&admitted, segment_policy())?;
    let current = FilesystemCatalogSnapshot::load(root, durable_fixture::policy()?)?;
    let expectation = CatalogPublicationExpectation::successor_of(&current.snapshot()?);
    let request = plan_recovery_next_head_finalization(&assessment, &candidate, expectation)?;
    let admission = FilesystemRecoveryAdmission::reopen(root)?;
    let mut finalizer =
        FilesystemRecoveryNextHeadFinalizer::open(admission, durable_fixture::policy()?)?;
    Ok(execute_recovery_next_head_finalization(
        &mut finalizer,
        request,
    ))
}

/// The canonical catalog-pool name a publication head selects.
pub(super) fn catalog_name(head: &ChecksummedPublicationHead<'_>) -> TestResult<String> {
    let mut name = format!("{:016x}-", head.generation().get());
    for byte in head.catalog_digest().as_bytes() {
        write!(name, "{byte:02x}")?;
    }
    name.push_str(".cat");
    Ok(name)
}

fn stage_path(root: &Path, stage: RecoveryStage) -> std::path::PathBuf {
    match stage {
        RecoveryStage::Segment => root.join("staging/current.seg"),
        RecoveryStage::Catalog => root.join("staging/current.cat"),
        RecoveryStage::NextHead => root.join("head.next"),
    }
}
