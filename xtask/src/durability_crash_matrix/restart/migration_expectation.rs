//! This module owns the independent expected state after migration process
//! death: the exact root inventory and the one lawful recovery plan.
//!
//! The rules here restate the recovery table in
//! `docs/formats/segment-store-v2/migration-recovery.md` without reading any
//! production classifier, so a planner defect cannot hide behind itself.

use std::collections::BTreeSet;

use keep::{StoreMigrationFixedStage, StoreMigrationPhase, StoreMigrationRecoveryPlan as Plan};
use xtask::{
    DurabilityCrashCase, DurabilityCrashCaseError, DurabilityCrashPoint, DurabilityCrashPosition,
};

use crate::durability_crash_matrix::DurabilityCrashMatrixError;
use crate::durability_crash_matrix::production_protocol::fixture::{
    CATALOG_POOL_PATH, SEGMENT_POOL_PATH,
};

pub(super) const INTENT_STAGE: &str = "migration.intent.next";
pub(super) const INTENT: &str = "migration.intent";
pub(super) const READER_LOCK: &str = "reader.lock";
pub(super) const MARKER_STAGE: &str = "FORMAT.next";
pub(super) const MARKER: &str = "FORMAT";
pub(super) const RECEIPT_STAGE: &str = "migration.receipt.next";
pub(super) const RECEIPT: &str = "migration.receipt";

/// The namespace prefix in admission order; each entry is one `during`
/// occurrence of `KEEP-CRASH-060`.
const PREFIX_DIRECTORIES: [&str; 6] = [
    "retention",
    "retention/roots",
    "retention/manifests",
    "gc",
    "recovery",
    "recovery/dispositions",
];

pub(super) struct MigrationExpectation {
    paths: BTreeSet<String>,
    plan: Plan,
}

impl MigrationExpectation {
    pub(super) fn for_case(case: DurabilityCrashCase) -> Result<Self, DurabilityCrashMatrixError> {
        let step = migration_step(case.point())?;
        let partial = partial_stage(case);
        let done = if case.position() == DurabilityCrashPosition::After
            || (case.position() == DurabilityCrashPosition::During && atomic(case.point()))
        {
            step.saturating_add(1)
        } else {
            step
        };
        let prefix_reached = prefix_reached(case)?;
        let mut paths = version_one_paths();
        if (1..5).contains(&done) {
            paths.insert(INTENT_STAGE.into());
        }
        if done >= 3 {
            paths.insert(INTENT.into());
        }
        if done >= 7 {
            paths.insert(READER_LOCK.into());
        }
        let directories = if done >= 8 {
            PREFIX_DIRECTORIES.len()
        } else {
            prefix_reached.unwrap_or(0)
        };
        paths.extend(
            PREFIX_DIRECTORIES
                .iter()
                .take(directories)
                .map(|path| (*path).into()),
        );
        if (10..14).contains(&done) {
            paths.insert(MARKER_STAGE.into());
        }
        if done >= 12 {
            paths.insert(MARKER.into());
        }
        if (16..20).contains(&done) {
            paths.insert(RECEIPT_STAGE.into());
        }
        if done >= 18 {
            paths.insert(RECEIPT.into());
        }
        if let Some(stage) = partial {
            paths.insert(stage_name(stage).into());
        }
        let plan = partial.map_or_else(
            || plan_after(done, prefix_reached),
            |stage| Plan::DiscardStage {
                stage,
                resume: write_phase(stage),
            },
        );
        Ok(Self { paths, plan })
    }

    pub(super) const fn paths(&self) -> &BTreeSet<String> {
        &self.paths
    }

    pub(super) const fn plan(&self) -> Plan {
        self.plan
    }
}

/// The exact version-1 store the migration starts from.
pub(super) fn version_one_paths() -> BTreeSet<String> {
    [
        "writer.lock",
        "staging",
        "segments",
        "catalogs",
        "HEAD",
        SEGMENT_POOL_PATH,
        CATALOG_POOL_PATH,
    ]
    .into_iter()
    .map(Into::into)
    .collect()
}

/// The exact root after one complete migration: no stage remains.
pub(super) fn complete_paths() -> BTreeSet<String> {
    let mut paths = version_one_paths();
    paths.extend(
        [INTENT, READER_LOCK, MARKER, RECEIPT]
            .into_iter()
            .chain(PREFIX_DIRECTORIES)
            .map(Into::into),
    );
    paths
}

fn migration_step(point: DurabilityCrashPoint) -> Result<usize, DurabilityCrashMatrixError> {
    DurabilityCrashPoint::MIGRATION
        .into_iter()
        .position(|candidate| candidate == point)
        .ok_or(DurabilityCrashMatrixError::PointSequenceMismatch { point })
}

/// Boundaries whose effect is one link, unlink, or exclusive creation, so
/// `during` cannot leave a partial effect.
const fn atomic(point: DurabilityCrashPoint) -> bool {
    matches!(
        point,
        DurabilityCrashPoint::MigrationLinkIntent
            | DurabilityCrashPoint::MigrationRemoveIntentStage
            | DurabilityCrashPoint::MigrationAdmitReaderFence
            | DurabilityCrashPoint::MigrationLinkMarker
            | DurabilityCrashPoint::MigrationRemoveMarkerStage
            | DurabilityCrashPoint::MigrationLinkReceipt
            | DurabilityCrashPoint::MigrationRemoveReceiptStage
    )
}

/// Process death during a stage write leaves an incomplete pre-effect stage.
fn partial_stage(case: DurabilityCrashCase) -> Option<StoreMigrationFixedStage> {
    if case.position() != DurabilityCrashPosition::During {
        return None;
    }
    match case.point() {
        DurabilityCrashPoint::MigrationWriteIntentStage => Some(StoreMigrationFixedStage::Intent),
        DurabilityCrashPoint::MigrationWriteMarkerStage => Some(StoreMigrationFixedStage::Marker),
        DurabilityCrashPoint::MigrationWriteReceiptStage => Some(StoreMigrationFixedStage::Receipt),
        _ => None,
    }
}

/// Process death during namespace admission leaves the first `occurrence + 1`
/// prefix directories, each with its parent synchronized.
fn prefix_reached(case: DurabilityCrashCase) -> Result<Option<usize>, DurabilityCrashMatrixError> {
    if case.point() != DurabilityCrashPoint::MigrationAdmitNamespacePrefix
        || case.position() != DurabilityCrashPosition::During
    {
        return Ok(None);
    }
    let point = case.point();
    let observed = case
        .occurrence()
        .ok_or(DurabilityCrashMatrixError::InvalidCase(
            DurabilityCrashCaseError::MissingOccurrence { point },
        ))?;
    let reached = observed
        .get()
        .checked_add(1)
        .ok_or(DurabilityCrashMatrixError::InvalidCase(
            DurabilityCrashCaseError::OccurrenceOutOfRange {
                point,
                observed,
                exclusive_limit: point.during_occurrences(),
            },
        ))?;
    usize::try_from(reached)
        .map(Some)
        .map_err(|source| DurabilityCrashMatrixError::Verification {
            phase: "represent reached migration namespace prefix",
            source: Box::new(source),
        })
}

const fn stage_name(stage: StoreMigrationFixedStage) -> &'static str {
    match stage {
        StoreMigrationFixedStage::Intent => INTENT_STAGE,
        StoreMigrationFixedStage::Marker => MARKER_STAGE,
        StoreMigrationFixedStage::Receipt => RECEIPT_STAGE,
    }
}

const fn write_phase(stage: StoreMigrationFixedStage) -> StoreMigrationPhase {
    match stage {
        StoreMigrationFixedStage::Intent => StoreMigrationPhase::WriteIntentStage,
        StoreMigrationFixedStage::Marker => StoreMigrationPhase::WriteMarkerStage,
        StoreMigrationFixedStage::Receipt => StoreMigrationPhase::WriteReceiptStage,
    }
}

/// The recovery-table row for a residue in which the first `done` phases
/// completed and nothing else happened: resume at the earliest phase the
/// residue cannot prove.
const fn plan_after(done: usize, prefix_reached: Option<usize>) -> Plan {
    let resume = match done {
        0 => return Plan::VersionOne,
        1 | 2 => StoreMigrationPhase::SynchronizeIntentStage,
        3 | 4 => StoreMigrationPhase::SynchronizeRootAfterIntent,
        5 | 6 => StoreMigrationPhase::SynchronizeRootAfterIntentCleanup,
        7 => match prefix_reached {
            Some(reached) if reached == PREFIX_DIRECTORIES.len() => {
                StoreMigrationPhase::SynchronizeRootAfterNamespace
            }
            _ => StoreMigrationPhase::AdmitNamespacePrefix,
        },
        8 | 9 => StoreMigrationPhase::SynchronizeRootAfterNamespace,
        10 | 11 => StoreMigrationPhase::SynchronizeMarkerStage,
        12 | 13 => StoreMigrationPhase::SynchronizeRootAfterMarker,
        14 | 15 => StoreMigrationPhase::SynchronizeRootAfterMarkerCleanup,
        16 | 17 => StoreMigrationPhase::SynchronizeReceiptStage,
        18 | 19 => StoreMigrationPhase::SynchronizeRootAfterReceipt,
        _ => return Plan::Complete,
    };
    Plan::Resume { resume }
}
