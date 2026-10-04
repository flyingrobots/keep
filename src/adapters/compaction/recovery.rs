//! This module owns recovery of an interrupted compaction successor on a
//! version-two root: the same three fixed stages the version-one recovery
//! protocols own, driven to one lawful state.

use std::error::Error;
use std::io;
use std::path::Path;

use crate::CatalogGeneration;
use crate::adapters::{
    CatalogRestartPolicy, FilesystemRecoveryStageDiscarder, RecoveryStage, RecoveryStageMetadata,
    admit_recovery_stage_bytes, fingerprint_recovery_stage,
};

/// What recovery found and did.
#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompactionRecovery {
    pub(super) discarded: Vec<RecoveryStage>,
    pub(super) finalized: Option<CatalogGeneration>,
}

impl CompactionRecovery {
    /// Stages whose discard completed with successful parent synchronization.
    #[must_use]
    pub fn discarded(&self) -> &[RecoveryStage] {
        &self.discarded
    }

    /// The generation a complete `head.next` was finalized to, if any.
    #[must_use]
    pub const fn finalized(&self) -> Option<CatalogGeneration> {
        self.finalized
    }

    /// Whether this report contains no completed recovery actions.
    ///
    /// On success this means no residue was found. In an error's progress report,
    /// it means no earlier action completed, not that the failed action had no effects.
    #[must_use]
    pub const fn was_idle(&self) -> bool {
        self.discarded.is_empty() && self.finalized.is_none()
    }
}

pub use super::recovery_failure::FilesystemCompactionRecoveryError;

pub(super) fn refused(
    phase: &'static str,
    source: impl Error + Send + Sync + 'static,
) -> FilesystemCompactionRecoveryError {
    FilesystemCompactionRecoveryError::refused(phase, source)
}

/// Recovers an interrupted compaction on the version-two root at
/// `store_root`, acquiring writer authority for the duration.
///
/// A retained `staging/current.seg` or `staging/current.cat` is discarded
/// (nothing published references it); a retained `head.next` is finalized
/// when it is complete and the exact successor of `HEAD`, and discarded
/// only when its assessment authorizes discard. Corruption refuses. Every
/// step retains its observed handle and rechecks exact bytes and identity
/// before execution; pathname mutation is not atomic against raw external edits.
///
/// # Allocation and I/O
///
/// Each stage is opened without following links and rejected above its format
/// length limit before materialization. Preflight retains bounded segment and
/// catalog bytes; cleanup may materialize one additional stage copy. Catalog
/// verification allocates separately under its format and caller policy limits.
/// These limits are not a total process memory cap. An execution error allocates
/// a bounded progress report containing at most the three fixed-stage completions.
///
/// # Errors
///
/// Returns [`FilesystemCompactionRecoveryError`] at the exact open,
/// assessment, planning, or execution failure. All residue is assessed and planned
/// before mutation begins. `execution()` is `None` for admission refusal; otherwise
/// it reports prior completed actions, the failed action and boundary, and known
/// or uncertain namespace effects. A failed sync is not rollback or a durability
/// receipt. Execution stops immediately; another call observes the store afresh.
pub fn recover_compaction(
    store_root: &Path,
    policy: CatalogRestartPolicy,
) -> Result<CompactionRecovery, FilesystemCompactionRecoveryError> {
    let discarder = FilesystemRecoveryStageDiscarder::open_version_two(store_root)
        .map_err(|source| refused("open", source))?;
    recover_with(discarder, policy, CompleteStageEvidence::Derivable)
}

#[cfg(test)]
pub(in crate::adapters) fn recover_compaction_unchecked_for_tests(
    store_root: &Path,
    policy: CatalogRestartPolicy,
) -> Result<CompactionRecovery, FilesystemCompactionRecoveryError> {
    let discarder =
        FilesystemRecoveryStageDiscarder::open_unchecked_version_two_for_tests(store_root)
            .map_err(|source| refused("open", source))?;
    recover_with(discarder, policy, CompleteStageEvidence::Derivable)
}

/// What proves a complete staged segment safe to discard.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::adapters) enum CompleteStageEvidence {
    /// Every staged record is named byte-identically by the current
    /// catalog: a compaction copy the next compaction reproduces.
    Derivable,
    /// No staged record is named by the current catalog: an ingestion
    /// stage that was never committed, so nothing published references it.
    Unpublished,
}

pub(in crate::adapters) fn recover_with(
    discarder: FilesystemRecoveryStageDiscarder,
    policy: CatalogRestartPolicy,
    evidence: CompleteStageEvidence,
) -> Result<CompactionRecovery, FilesystemCompactionRecoveryError> {
    recover_after_preflight(discarder, policy, evidence, || Ok(()))
}

/// The callback supplies a deterministic observation/execution interleaving.
pub(super) fn recover_after_preflight(
    discarder: FilesystemRecoveryStageDiscarder,
    policy: CatalogRestartPolicy,
    evidence: CompleteStageEvidence,
    after_preflight: impl FnOnce() -> io::Result<()>,
) -> Result<CompactionRecovery, FilesystemCompactionRecoveryError> {
    recover_scheduled(discarder, policy, evidence, after_preflight, &mut |_, _| {
        Ok(())
    })
}

pub(super) fn recover_scheduled(
    discarder: FilesystemRecoveryStageDiscarder,
    policy: CatalogRestartPolicy,
    evidence: CompleteStageEvidence,
    after_preflight: impl FnOnce() -> io::Result<()>,
    schedule: &mut super::recovery_schedule::Schedule<'_>,
) -> Result<CompactionRecovery, FilesystemCompactionRecoveryError> {
    let mut plan = super::recovery_preflight::prepare(&discarder, policy, evidence)?;
    after_preflight().map_err(|source| refused("observation interleaving", source))?;
    plan.verify(&discarder)?;
    super::recovery_execution::execute(discarder, policy, plan, schedule)
}

pub(super) fn admitted_stage(
    stage: RecoveryStage,
    bytes: &[u8],
) -> Result<crate::adapters::AdmittedRecoveryStageBytes<'_>, FilesystemCompactionRecoveryError> {
    let length = u64::try_from(bytes.len()).map_err(|source| refused("stage length", source))?;
    let metadata = RecoveryStageMetadata::new(stage, length)
        .map_err(|source| refused("stage metadata", source))?;
    let evidence = fingerprint_recovery_stage(metadata, bytes)
        .map_err(|source| refused("fingerprint stage", source))?;
    admit_recovery_stage_bytes(stage, evidence, bytes)
        .map_err(|source| refused("admit stage bytes", source))
}
