//! This module owns effect-free admission of the complete compaction recovery queue.

use super::recovery::{
    CompleteStageEvidence, FilesystemCompactionRecoveryError as Error, admitted_stage, refused,
};
use super::recovery_observation::{ObservedStage, read_stage};
use crate::adapters::{
    AdmittedSegment, CatalogPublicationExpectation, CatalogRestartError, CatalogRestartPhase,
    CatalogRestartPolicy, ChecksummedCatalog, FilesystemRecoveryStageDiscarder,
    RecoveryCatalogStage, RecoveryNextHeadFinalizationRequest, RecoveryNextHeadStage,
    RecoverySegmentStage, RecoveryStage, RecoveryStageAssessment, RecoveryStageDiscardRequest,
    RecoveryStageParent, assess_recovery_stage, catalog_restart_loader,
    plan_recovery_next_head_finalization, plan_recovery_stage_discard,
};
use cap_std::fs::Dir;
use std::io;

pub(super) struct RecoveryPlan {
    pub(super) stages: Vec<PreparedStage>,
    pub(super) next: Option<PreparedNextHead>,
}

pub(super) struct PreparedStage {
    pub(super) observed: ObservedStage,
    pub(super) action: StageAction,
}

pub(super) enum StageAction {
    Derivable,
    Truncated(RecoveryStageDiscardRequest),
}

pub(super) struct PreparedNextHead {
    pub(super) observed: ObservedStage,
    pub(super) action: NextHeadAction,
}

pub(super) enum NextHeadAction {
    Discard(RecoveryStageDiscardRequest),
    Finalize(RecoveryNextHeadFinalizationRequest),
}

pub(super) fn prepare(
    discarder: &FilesystemRecoveryStageDiscarder,
    policy: CatalogRestartPolicy,
    evidence: CompleteStageEvidence,
) -> Result<RecoveryPlan, Error> {
    let mut stages = Vec::new();
    for stage in [RecoveryStage::Segment, RecoveryStage::Catalog] {
        if let Some(observed) = read_stage(discarder, stage)? {
            let action = staging_action(discarder, stage, observed.bytes(), policy, evidence)?;
            stages.push(PreparedStage { observed, action });
        }
    }
    let next = read_stage(discarder, RecoveryStage::NextHead)?
        .map(|observed| {
            let action = next_head_action(discarder, policy, observed.bytes())?;
            Ok::<_, Error>(PreparedNextHead { observed, action })
        })
        .transpose()?;
    Ok(RecoveryPlan { stages, next })
}

fn next_head_action(
    discarder: &FilesystemRecoveryStageDiscarder,
    policy: CatalogRestartPolicy,
    bytes: &[u8],
) -> Result<NextHeadAction, Error> {
    let admitted = admitted_stage(RecoveryStage::NextHead, bytes)?;
    let assessment = assess_recovery_stage(&admitted, policy.segment_read())
        .map_err(|source| refused("assess head.next", source))?;
    if !matches!(
        assessment,
        RecoveryStageAssessment::NextHead {
            state: RecoveryNextHeadStage::Complete(_),
            ..
        }
    ) {
        return plan_recovery_stage_discard(&assessment)
            .map(NextHeadAction::Discard)
            .map_err(|source| refused("plan head.next discard", source));
    }
    let root = root_directory(discarder);
    let candidate = catalog_restart_loader::load_from_directory(root, "head.next", policy)
        .map_err(|source| refused("load head.next successor", source))?;
    let snapshot = candidate
        .snapshot()
        .map_err(|source| refused("admit head.next successor", source))?;
    let expectation = current_expectation(root, policy)?;
    plan_recovery_next_head_finalization(&assessment, &snapshot, expectation)
        .map(NextHeadAction::Finalize)
        .map_err(|source| refused("plan head.next finalization", source))
}

fn current_expectation(
    root: &Dir,
    policy: CatalogRestartPolicy,
) -> Result<CatalogPublicationExpectation, Error> {
    match catalog_restart_loader::load_from_directory(root, "HEAD", policy) {
        Ok(current) => {
            let snapshot = current
                .snapshot()
                .map_err(|source| refused("admit current catalog", source))?;
            Ok(CatalogPublicationExpectation::successor_of(&snapshot))
        }
        Err(CatalogRestartError::Io {
            phase: CatalogRestartPhase::OpenHead,
            source,
        }) if source.kind() == io::ErrorKind::NotFound => {
            Ok(CatalogPublicationExpectation::uninitialized())
        }
        Err(source) => Err(refused("load current catalog", source)),
    }
}

fn staging_action(
    discarder: &FilesystemRecoveryStageDiscarder,
    stage: RecoveryStage,
    bytes: &[u8],
    policy: CatalogRestartPolicy,
    evidence: CompleteStageEvidence,
) -> Result<StageAction, Error> {
    let admitted = admitted_stage(stage, bytes)?;
    let assessment = assess_recovery_stage(&admitted, policy.segment_read())
        .map_err(|source| refused("assess stage", source))?;
    match &assessment {
        RecoveryStageAssessment::Segment {
            state: RecoverySegmentStage::Complete(segment),
            ..
        } => match evidence {
            CompleteStageEvidence::Derivable => {
                require_derivable_segment(discarder, segment, policy)?;
            }
            CompleteStageEvidence::Unpublished => {
                require_unpublished_segment(discarder, segment, policy)?;
            }
        },
        RecoveryStageAssessment::Segment {
            state: RecoverySegmentStage::Reusable(_),
            ..
        } => {}
        RecoveryStageAssessment::Catalog {
            state: RecoveryCatalogStage::Complete(catalog),
            ..
        } => {
            require_successor_candidate(discarder, catalog, policy)?;
        }
        _ => {
            return plan_recovery_stage_discard(&assessment)
                .map(StageAction::Truncated)
                .map_err(|source| refused("plan stage discard", source));
        }
    }
    Ok(StageAction::Derivable)
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
) -> Result<(), Error> {
    let current =
        catalog_restart_loader::load_from_directory(root_directory(discarder), "HEAD", policy)
            .map_err(|source| refused("load current catalog", source))?;
    let snapshot = current
        .snapshot()
        .map_err(|source| refused("admit current catalog", source))?;
    for record in segment.records() {
        let record = record.map_err(|source| refused("reread staged record", source))?;
        let named = snapshot.record(record.identity()).ok_or_else(|| {
            refused(
                "derivable segment",
                super::CompactionRecoveryRefusal::RecordNotNamed {
                    identity: record.identity(),
                },
            )
        })?;
        if named.header() != record.header()
            || named.payload() != record.payload()
            || named.checksum() != record.checksum()
        {
            return Err(refused(
                "derivable segment",
                super::CompactionRecoveryRefusal::RecordMismatch {
                    identity: record.identity(),
                },
            ));
        }
    }
    Ok(())
}

/// A complete staged segment is unpublished when the current catalog names
/// none of its records: an ingestion stage that never reached commit.
fn require_unpublished_segment(
    discarder: &FilesystemRecoveryStageDiscarder,
    segment: &AdmittedSegment<'_>,
    policy: CatalogRestartPolicy,
) -> Result<(), Error> {
    let current =
        catalog_restart_loader::load_from_directory(root_directory(discarder), "HEAD", policy)
            .map_err(|source| refused("load current catalog", source))?;
    let snapshot = current
        .snapshot()
        .map_err(|source| refused("admit current catalog", source))?;
    for record in segment.records() {
        let record = record.map_err(|source| refused("reread staged record", source))?;
        if snapshot.record(record.identity()).is_some() {
            return Err(refused(
                "unpublished segment",
                super::CompactionRecoveryRefusal::RecordAlreadyNamed {
                    identity: record.identity(),
                },
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
) -> Result<(), Error> {
    let current =
        catalog_restart_loader::load_from_directory(root_directory(discarder), "HEAD", policy)
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
            super::CompactionRecoveryRefusal::SuccessorMismatch {
                expected_generation: successor,
                observed_generation: catalog.generation(),
                expected_predecessor: current.catalog_digest(),
                observed_predecessor: catalog.previous_catalog_digest(),
            },
        ))
    }
}

impl RecoveryPlan {
    pub(super) fn verify(
        &mut self,
        discarder: &FilesystemRecoveryStageDiscarder,
    ) -> Result<(), Error> {
        for stage in &mut self.stages {
            stage.observed.verify(discarder)?;
        }
        if let Some(next) = &mut self.next {
            next.observed.verify(discarder)?;
        }
        Ok(())
    }
}
