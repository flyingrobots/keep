//! This module owns recovery of an interrupted compaction successor on a
//! version-two root: the same three fixed stages the version-one recovery
//! protocols own, driven to one lawful state.

use std::error::Error;
use std::fmt;
use std::io;
use std::path::Path;

use crate::CatalogGeneration;
use crate::adapters::filesystem_catalog_artifact::synchronize_directory;
use crate::adapters::filesystem_exact_record as exact_record;
use crate::adapters::{
    CatalogRestartPolicy, FilesystemRecoveryNextHeadFinalizer, FilesystemRecoveryStageDiscarder,
    RecoveryStage, RecoveryStageMetadata, RecoveryStageParent, admit_recovery_stage_bytes,
    execute_recovery_next_head_finalization, execute_recovery_stage_discard,
    fingerprint_recovery_stage,
};

/// What recovery found and did.
#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompactionRecovery {
    discarded: Vec<RecoveryStage>,
    finalized: Option<CatalogGeneration>,
}

impl CompactionRecovery {
    /// The stages discarded, in the order examined.
    #[must_use]
    pub fn discarded(&self) -> &[RecoveryStage] {
        &self.discarded
    }

    /// The generation a complete `head.next` was finalized to, if any.
    #[must_use]
    pub const fn finalized(&self) -> Option<CatalogGeneration> {
        self.finalized
    }

    /// Whether the store held no residue at all.
    #[must_use]
    pub const fn was_idle(&self) -> bool {
        self.discarded.is_empty() && self.finalized.is_none()
    }
}

/// Why recovery could not complete. Planning failures precede all recovery effects.
/// Execution failures may follow changes; returning an error is not rollback.
#[derive(Debug)]
pub struct FilesystemCompactionRecoveryError {
    phase: &'static str,
    source: Box<dyn Error + Send + Sync>,
}

impl fmt::Display for FilesystemCompactionRecoveryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "compaction recovery failed at {}", self.phase)
    }
}

impl Error for FilesystemCompactionRecoveryError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.source.as_ref())
    }
}

pub(super) fn refused(
    phase: &'static str,
    source: impl Error + Send + Sync + 'static,
) -> FilesystemCompactionRecoveryError {
    FilesystemCompactionRecoveryError::refused(phase, source)
}

impl FilesystemCompactionRecoveryError {
    pub(in crate::adapters) fn refused(
        phase: &'static str,
        source: impl Error + Send + Sync + 'static,
    ) -> Self {
        Self {
            phase,
            source: Box::new(source),
        }
    }
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
/// These limits are not a total process memory cap.
///
/// # Errors
///
/// Returns [`FilesystemCompactionRecoveryError`] at the exact open,
/// assessment, planning, or execution failure. All residue is assessed and planned
/// before mutation begins. A later execution error does not roll back prior effects.
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
    mut discarder: FilesystemRecoveryStageDiscarder,
    policy: CatalogRestartPolicy,
    evidence: CompleteStageEvidence,
    after_preflight: impl FnOnce() -> io::Result<()>,
) -> Result<CompactionRecovery, FilesystemCompactionRecoveryError> {
    let mut plan = super::recovery_preflight::prepare(&discarder, policy, evidence)?;
    after_preflight().map_err(|source| refused("observation interleaving", source))?;
    plan.verify(&discarder)?;
    let mut recovery = CompactionRecovery {
        discarded: Vec::new(),
        finalized: None,
    };
    for mut stage in plan.stages {
        match stage.action {
            super::recovery_preflight::StageAction::Derivable => {
                discard_derivable(&discarder, &mut stage.observed)?;
            }
            super::recovery_preflight::StageAction::Truncated(request) => {
                stage.observed.verify(&discarder)?;
                let _receipt = execute_recovery_stage_discard(&mut discarder, request)
                    .map_err(|source| refused("discard stage", source))?;
            }
        }
        recovery.discarded.push(stage.observed.stage());
    }
    let Some(mut next) = plan.next else {
        return Ok(recovery);
    };
    next.observed.verify(&discarder)?;
    match next.action {
        super::recovery_preflight::NextHeadAction::Discard(request) => {
            let _receipt = execute_recovery_stage_discard(&mut discarder, request)
                .map_err(|source| refused("discard head.next", source))?;
            recovery.discarded.push(RecoveryStage::NextHead);
        }
        super::recovery_preflight::NextHeadAction::Finalize(request) => {
            let mut finalizer = FilesystemRecoveryNextHeadFinalizer { discarder, policy };
            let _receipt = execute_recovery_next_head_finalization(&mut finalizer, request)
                .map_err(|source| refused("finalize head.next", source))?;
            recovery.finalized = Some(request.target().generation());
        }
    }
    Ok(recovery)
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

/// Unlinks one derivable stage only while its bytes are exactly as assessed,
/// and synchronizes `staging`.
fn discard_derivable(
    discarder: &FilesystemRecoveryStageDiscarder,
    observed: &mut super::recovery_observation::ObservedStage,
) -> Result<(), FilesystemCompactionRecoveryError> {
    let stage = observed.stage();
    let staging = discarder
        .inventory
        .parent_directory(RecoveryStageParent::Staging);
    observed.verify(discarder)?;
    staging
        .remove_file(stage.file_name())
        .map_err(|source| refused("discard stage", source))?;
    exact_record::require_absent(staging, stage.file_name())
        .map_err(|source| refused("discard stage", io::Error::other(source)))?;
    synchronize_directory(staging).map_err(|source| refused("synchronize staging", source))
}
