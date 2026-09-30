//! This module owns recovery of an interrupted compaction successor on a
//! version-two root: the same three fixed stages the version-one recovery
//! protocols own, driven to one lawful state.

use std::error::Error;
use std::fmt;
use std::io;
use std::path::Path;

use cap_std::fs::Dir;

use crate::CatalogGeneration;
use crate::adapters::filesystem_catalog_artifact::synchronize_directory;
use crate::adapters::filesystem_exact_record as exact_record;
use crate::adapters::{
    AdmittedSegment, CatalogPublicationExpectation, CatalogRestartPolicy, ChecksummedCatalog,
    FilesystemRecoveryNextHeadFinalizer, FilesystemRecoveryStageDiscarder, RecoveryCatalogStage,
    RecoveryNextHeadStage, RecoverySegmentStage, RecoveryStage, RecoveryStageAssessment,
    RecoveryStageMetadata, RecoveryStageParent, admit_recovery_stage_bytes, assess_recovery_stage,
    catalog_restart_loader, execute_recovery_next_head_finalization,
    execute_recovery_stage_discard, fingerprint_recovery_stage,
    plan_recovery_next_head_finalization, plan_recovery_stage_discard,
};

const SEGMENT_STAGE: &str = "current.seg";
const CATALOG_STAGE: &str = "current.cat";
const NEXT_HEAD: &str = "head.next";
const HEAD: &str = "HEAD";

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

/// Why recovery refused; the store is left as found.
#[derive(Debug)]
pub struct FilesystemCompactionRecoveryError {
    phase: &'static str,
    source: Box<dyn Error + Send + Sync>,
}

impl fmt::Display for FilesystemCompactionRecoveryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "compaction recovery refused at {}", self.phase)
    }
}

impl Error for FilesystemCompactionRecoveryError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.source.as_ref())
    }
}

fn refused(
    phase: &'static str,
    source: impl Error + Send + Sync + 'static,
) -> FilesystemCompactionRecoveryError {
    FilesystemCompactionRecoveryError {
        phase,
        source: Box::new(source),
    }
}

/// Recovers an interrupted compaction on the version-two root at
/// `store_root`, acquiring writer authority for the duration.
///
/// A retained `staging/current.seg` or `staging/current.cat` is discarded
/// (nothing published references it); a retained `head.next` is finalized
/// when it is complete and the exact successor of `HEAD`, and discarded
/// otherwise. Every step runs the version-one recovery protocol it belongs
/// to, with its evidence binding.
///
/// # Errors
///
/// Returns [`FilesystemCompactionRecoveryError`] at the exact open,
/// assessment, planning, or execution refusal.
pub fn recover_compaction(
    store_root: &Path,
    policy: CatalogRestartPolicy,
) -> Result<CompactionRecovery, FilesystemCompactionRecoveryError> {
    let discarder = FilesystemRecoveryStageDiscarder::open_version_two(store_root)
        .map_err(|source| refused("open", source))?;
    recover_with(discarder, policy)
}

#[cfg(test)]
pub(in crate::adapters) fn recover_compaction_unchecked_for_tests(
    store_root: &Path,
    policy: CatalogRestartPolicy,
) -> Result<CompactionRecovery, FilesystemCompactionRecoveryError> {
    let discarder =
        FilesystemRecoveryStageDiscarder::open_unchecked_version_two_for_tests(store_root)
            .map_err(|source| refused("open", source))?;
    recover_with(discarder, policy)
}

fn recover_with(
    mut discarder: FilesystemRecoveryStageDiscarder,
    policy: CatalogRestartPolicy,
) -> Result<CompactionRecovery, FilesystemCompactionRecoveryError> {
    let mut recovery = CompactionRecovery {
        discarded: Vec::new(),
        finalized: None,
    };
    for (stage, name) in [
        (RecoveryStage::Segment, SEGMENT_STAGE),
        (RecoveryStage::Catalog, CATALOG_STAGE),
    ] {
        if let Some(bytes) = read_stage(&discarder, RecoveryStageParent::Staging, name)? {
            resolve_staging(&mut discarder, stage, name, &bytes, policy)?;
            recovery.discarded.push(stage);
        }
    }
    let Some(bytes) = read_stage(&discarder, RecoveryStageParent::Root, NEXT_HEAD)? else {
        return Ok(recovery);
    };
    let admitted = admitted_stage(RecoveryStage::NextHead, &bytes)?;
    let assessment = assess_recovery_stage(&admitted, policy.segment_read())
        .map_err(|source| refused("assess head.next", source))?;
    let complete = matches!(
        assessment,
        RecoveryStageAssessment::NextHead {
            state: RecoveryNextHeadStage::Complete(_),
            ..
        }
    );
    if !complete {
        let request = plan_recovery_stage_discard(&assessment)
            .map_err(|source| refused("plan head.next discard", source))?;
        let _receipt = execute_recovery_stage_discard(&mut discarder, request)
            .map_err(|source| refused("discard head.next", source))?;
        recovery.discarded.push(RecoveryStage::NextHead);
        return Ok(recovery);
    }
    let root = discarder
        .inventory
        .parent_directory(RecoveryStageParent::Root)
        .try_clone()
        .map_err(|source| refused("clone root", source))?;
    let candidate = catalog_restart_loader::load_from_directory(&root, NEXT_HEAD, policy)
        .map_err(|source| refused("load head.next successor", source))?;
    let candidate_snapshot = candidate
        .snapshot()
        .map_err(|source| refused("admit head.next successor", source))?;
    let expectation = match catalog_restart_loader::load_from_directory(&root, HEAD, policy) {
        Ok(current) => {
            let snapshot = current
                .snapshot()
                .map_err(|source| refused("admit current catalog", source))?;
            CatalogPublicationExpectation::successor_of(&snapshot)
        }
        Err(_absent) => CatalogPublicationExpectation::uninitialized(),
    };
    let request =
        plan_recovery_next_head_finalization(&assessment, &candidate_snapshot, expectation)
            .map_err(|source| refused("plan head.next finalization", source))?;
    let mut finalizer = FilesystemRecoveryNextHeadFinalizer { discarder, policy };
    let _receipt = execute_recovery_next_head_finalization(&mut finalizer, request)
        .map_err(|source| refused("finalize head.next", source))?;
    recovery.finalized = Some(candidate_snapshot.generation());
    Ok(recovery)
}

fn read_stage(
    discarder: &FilesystemRecoveryStageDiscarder,
    parent: RecoveryStageParent,
    name: &str,
) -> Result<Option<Vec<u8>>, FilesystemCompactionRecoveryError> {
    let directory = discarder.inventory.parent_directory(parent);
    let mut file = match crate::adapters::filesystem_exact_record::open_read(directory, name) {
        Ok(file) => file,
        Err(source) if source.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(source) => return Err(refused("open stage", source)),
    };
    let mut bytes = Vec::new();
    std::io::Read::read_to_end(&mut file, &mut bytes)
        .map_err(|source| refused("read stage", source))?;
    Ok(Some(bytes))
}

fn admitted_stage(
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

/// Resolves one staging residue: a truncated stage through the version-one
/// discard protocol; a complete or reusable stage through compaction's own
/// evidence-bound discard, after proving it carries nothing the current
/// catalog does not already name.
fn resolve_staging(
    discarder: &mut FilesystemRecoveryStageDiscarder,
    stage: RecoveryStage,
    name: &str,
    bytes: &[u8],
    policy: CatalogRestartPolicy,
) -> Result<(), FilesystemCompactionRecoveryError> {
    let admitted = admitted_stage(stage, bytes)?;
    let assessment = assess_recovery_stage(&admitted, policy.segment_read())
        .map_err(|source| refused("assess stage", source))?;
    let derivable = match &assessment {
        RecoveryStageAssessment::Segment {
            state: RecoverySegmentStage::Complete(segment),
            ..
        } => {
            require_derivable_segment(discarder, segment, policy)?;
            true
        }
        RecoveryStageAssessment::Segment {
            state: RecoverySegmentStage::Reusable(_),
            ..
        } => true,
        RecoveryStageAssessment::Catalog {
            state: RecoveryCatalogStage::Complete(catalog),
            ..
        } => {
            require_successor_candidate(discarder, catalog, policy)?;
            true
        }
        _ => false,
    };
    if derivable {
        return discard_derivable(discarder, name, bytes);
    }
    let request = plan_recovery_stage_discard(&assessment)
        .map_err(|source| refused("plan stage discard", source))?;
    let _receipt = execute_recovery_stage_discard(discarder, request)
        .map_err(|source| refused("discard stage", source))?;
    Ok(())
}

const fn root_directory(discarder: &FilesystemRecoveryStageDiscarder) -> &Dir {
    discarder
        .inventory
        .parent_directory(RecoveryStageParent::Root)
}

/// A complete staged segment is derivable when the current catalog names
/// every record it holds with byte-identical content: it is a copy the next
/// compaction reproduces exactly.
fn require_derivable_segment(
    discarder: &FilesystemRecoveryStageDiscarder,
    segment: &AdmittedSegment<'_>,
    policy: CatalogRestartPolicy,
) -> Result<(), FilesystemCompactionRecoveryError> {
    let current =
        catalog_restart_loader::load_from_directory(root_directory(discarder), HEAD, policy)
            .map_err(|source| refused("load current catalog", source))?;
    let snapshot = current
        .snapshot()
        .map_err(|source| refused("admit current catalog", source))?;
    for record in segment.records() {
        let record = record.map_err(|source| refused("reread staged record", source))?;
        let named = snapshot.record(record.identity()).ok_or_else(|| {
            refused(
                "derivable segment",
                io::Error::other("a staged record is not named"),
            )
        })?;
        if named.header() != record.header()
            || named.payload() != record.payload()
            || named.checksum() != record.checksum()
        {
            return Err(refused(
                "derivable segment",
                io::Error::other("a staged record differs from the named record"),
            ));
        }
    }
    Ok(())
}

/// A complete staged catalog is derivable when it is exactly the successor
/// candidate of the current head: never published, reproduced by the next
/// compaction.
fn require_successor_candidate(
    discarder: &FilesystemRecoveryStageDiscarder,
    catalog: &ChecksummedCatalog<'_>,
    policy: CatalogRestartPolicy,
) -> Result<(), FilesystemCompactionRecoveryError> {
    let current =
        catalog_restart_loader::load_from_directory(root_directory(discarder), HEAD, policy)
            .map_err(|source| refused("load current catalog", source))?;
    let successor = current
        .generation()
        .successor()
        .map_err(|source| refused("successor generation", source))?;
    if catalog.generation() == successor
        && catalog.previous_catalog_digest() == Some(current.catalog_digest())
    {
        Ok(())
    } else {
        Err(refused(
            "successor candidate",
            io::Error::other("the staged catalog is not the current head's successor"),
        ))
    }
}

/// Unlinks one derivable stage only while its bytes are exactly as assessed,
/// and synchronizes `staging`.
fn discard_derivable(
    discarder: &FilesystemRecoveryStageDiscarder,
    name: &str,
    expected: &[u8],
) -> Result<(), FilesystemCompactionRecoveryError> {
    let staging = discarder
        .inventory
        .parent_directory(RecoveryStageParent::Staging);
    let observed = read_stage(discarder, RecoveryStageParent::Staging, name)?
        .ok_or_else(|| refused("discard stage", io::Error::other("the stage vanished")))?;
    if observed != expected {
        return Err(refused(
            "discard stage",
            io::Error::other("the stage changed after assessment"),
        ));
    }
    staging
        .remove_file(name)
        .map_err(|source| refused("discard stage", source))?;
    exact_record::require_absent(staging, name)
        .map_err(|source| refused("discard stage", io::Error::other(source.to_string())))?;
    synchronize_directory(staging).map_err(|source| refused("synchronize staging", source))
}
