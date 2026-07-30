//! This module owns expected states for each durable protocol sequence.

use std::collections::{BTreeMap, BTreeSet};

use super::steps::{
    catalog_bytes, catalog_step, completed_step, head_bytes, head_step, initialization_step,
    interrupted_segment_bytes, segment_bytes, segment_step,
};
use super::{
    ArtifactBytes, CATALOG_STAGE, CATALOGS, ExpectedStoreState, HEAD, NEXT_HEAD, SEGMENT_STAGE,
    SEGMENTS, STAGING, WRITER_LOCK,
};
use crate::durability_crash_matrix::DurabilityCrashMatrixError;
use crate::durability_crash_matrix::production_protocol::fixture::{
    CATALOG_POOL_PATH, SEGMENT_POOL_PATH,
};
use xtask::{DurabilityCrashCase, DurabilityCrashPoint, DurabilityCrashPosition};

pub(super) fn segment(
    case: DurabilityCrashCase,
) -> Result<ExpectedStoreState, DurabilityCrashMatrixError> {
    let mut state = ExpectedStoreState::initialized();
    let completed = completed_step(segment_step(case.point())?, case.position(), case.point());
    if let Some(bytes) = interrupted_segment_bytes(case) {
        state.artifacts.insert(SEGMENT_STAGE, bytes);
    } else if let Some(step) = completed {
        if step <= 9 {
            state.artifacts.insert(SEGMENT_STAGE, segment_bytes(step));
        }
        if step >= 8 {
            state
                .artifacts
                .insert(SEGMENT_POOL_PATH, ArtifactBytes::Segment(337));
        }
        if (8..=9).contains(&step) {
            state.hard_link = Some((SEGMENT_STAGE, SEGMENT_POOL_PATH));
        }
    }
    Ok(state)
}

pub(super) fn catalog(
    case: DurabilityCrashCase,
) -> Result<ExpectedStoreState, DurabilityCrashMatrixError> {
    let mut state = ExpectedStoreState::initialized();
    state
        .artifacts
        .insert(SEGMENT_POOL_PATH, ArtifactBytes::Segment(337));
    let completed = completed_step(catalog_step(case.point())?, case.position(), case.point());
    if case.point() == DurabilityCrashPoint::WriteCatalog
        && case.position() == DurabilityCrashPosition::During
    {
        state
            .artifacts
            .insert(CATALOG_STAGE, ArtifactBytes::Catalog(176));
    } else if let Some(step) = completed {
        if step <= 5 {
            state.artifacts.insert(CATALOG_STAGE, catalog_bytes(step));
        }
        if step >= 4 {
            state
                .artifacts
                .insert(CATALOG_POOL_PATH, ArtifactBytes::Catalog(352));
        }
        if (4..=5).contains(&step) {
            state.hard_link = Some((CATALOG_STAGE, CATALOG_POOL_PATH));
        }
    }
    Ok(state)
}

pub(super) fn head(
    case: DurabilityCrashCase,
) -> Result<ExpectedStoreState, DurabilityCrashMatrixError> {
    let mut state = ExpectedStoreState::initialized();
    state
        .artifacts
        .insert(SEGMENT_POOL_PATH, ArtifactBytes::Segment(337));
    state
        .artifacts
        .insert(CATALOG_POOL_PATH, ArtifactBytes::Catalog(352));
    let completed = completed_step(head_step(case.point())?, case.position(), case.point());
    if case.point() == DurabilityCrashPoint::WriteHead
        && case.position() == DurabilityCrashPosition::During
    {
        state.artifacts.insert(NEXT_HEAD, ArtifactBytes::Head(64));
    } else if let Some(step) = completed {
        if step <= 3 {
            state.artifacts.insert(NEXT_HEAD, head_bytes(step));
        } else {
            state.artifacts.insert(HEAD, ArtifactBytes::Head(128));
        }
    }
    Ok(state)
}

pub(super) fn recovery(
    case: DurabilityCrashCase,
) -> Result<ExpectedStoreState, DurabilityCrashMatrixError> {
    let mut state = ExpectedStoreState::initialized();
    match case.point() {
        DurabilityCrashPoint::RemoveRecoveryStage
            if case.position() == DurabilityCrashPosition::Before =>
        {
            state
                .artifacts
                .insert(SEGMENT_STAGE, ArtifactBytes::Segment(32));
        }
        DurabilityCrashPoint::RemoveRecoveryHead
            if case.position() == DurabilityCrashPosition::Before =>
        {
            state.artifacts.insert(NEXT_HEAD, ArtifactBytes::Head(64));
        }
        DurabilityCrashPoint::RemoveRecoveryStage
        | DurabilityCrashPoint::SynchronizeStagingAfterRecovery
        | DurabilityCrashPoint::RemoveRecoveryHead
        | DurabilityCrashPoint::SynchronizeRootAfterRecovery => {}
        point => return Err(DurabilityCrashMatrixError::PointSequenceMismatch { point }),
    }
    Ok(state)
}

pub(super) fn initialization(
    case: DurabilityCrashCase,
) -> Result<ExpectedStoreState, DurabilityCrashMatrixError> {
    let step = initialization_step(case.point())?;
    let completed = completed_step(step, case.position(), case.point());
    let mut state = ExpectedStoreState {
        directories: BTreeSet::new(),
        artifacts: BTreeMap::new(),
        hard_link: None,
    };
    if let Some(completed) = completed {
        state.artifacts.insert(WRITER_LOCK, ArtifactBytes::Empty);
        if completed >= 1 {
            state.directories.insert(STAGING);
        }
        if completed >= 2 {
            state.directories.insert(SEGMENTS);
        }
        if completed >= 3 {
            state.directories.insert(CATALOGS);
        }
    }
    Ok(state)
}
