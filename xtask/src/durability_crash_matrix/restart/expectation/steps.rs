//! This module owns independent crash-point ordering and byte-state rules.

use super::ArtifactBytes;
use crate::durability_crash_matrix::DurabilityCrashMatrixError;
use xtask::{DurabilityCrashCase, DurabilityCrashPoint, DurabilityCrashPosition};

pub(super) fn completed_step(
    step: usize,
    position: DurabilityCrashPosition,
    point: DurabilityCrashPoint,
) -> Option<usize> {
    if position == DurabilityCrashPosition::After
        || (position == DurabilityCrashPosition::During && atomic(point))
    {
        Some(step)
    } else {
        step.checked_sub(1)
    }
}

const fn atomic(point: DurabilityCrashPoint) -> bool {
    matches!(
        point,
        DurabilityCrashPoint::CreateSegmentStage
            | DurabilityCrashPoint::LinkSegment
            | DurabilityCrashPoint::RemoveSegmentStage
            | DurabilityCrashPoint::CreateCatalogStage
            | DurabilityCrashPoint::LinkCatalog
            | DurabilityCrashPoint::RemoveCatalogStage
            | DurabilityCrashPoint::CreateHeadStage
            | DurabilityCrashPoint::ReplaceHead
            | DurabilityCrashPoint::RemoveRecoveryStage
            | DurabilityCrashPoint::RemoveRecoveryHead
            | DurabilityCrashPoint::OpenAndLockWriterFile
            | DurabilityCrashPoint::CreateStagingDirectory
            | DurabilityCrashPoint::CreateSegmentPoolDirectory
            | DurabilityCrashPoint::CreateCatalogPoolDirectory
    )
}

pub(super) fn interrupted_segment_bytes(case: DurabilityCrashCase) -> Option<ArtifactBytes> {
    if case.position() != DurabilityCrashPosition::During {
        return None;
    }
    match case.point() {
        DurabilityCrashPoint::WriteSegmentHeader => Some(ArtifactBytes::Segment(32)),
        DurabilityCrashPoint::AppendSegmentRecord => Some(ArtifactBytes::Segment(136)),
        DurabilityCrashPoint::AppendSegmentSeal => Some(ArtifactBytes::Segment(273)),
        _ => None,
    }
}

pub(super) const fn segment_bytes(step: usize) -> ArtifactBytes {
    match step {
        0 => ArtifactBytes::Empty,
        1 => ArtifactBytes::Segment(64),
        2..=4 => ArtifactBytes::Segment(209),
        _ => ArtifactBytes::Segment(337),
    }
}

pub(super) const fn catalog_bytes(step: usize) -> ArtifactBytes {
    if step == 0 {
        ArtifactBytes::Empty
    } else {
        ArtifactBytes::Catalog(352)
    }
}

pub(super) const fn head_bytes(step: usize) -> ArtifactBytes {
    if step == 0 {
        ArtifactBytes::Empty
    } else {
        ArtifactBytes::Head(128)
    }
}

pub(super) fn segment_step(
    point: DurabilityCrashPoint,
) -> Result<usize, DurabilityCrashMatrixError> {
    const POINTS: [DurabilityCrashPoint; 12] = [
        DurabilityCrashPoint::CreateSegmentStage,
        DurabilityCrashPoint::WriteSegmentHeader,
        DurabilityCrashPoint::AppendSegmentRecord,
        DurabilityCrashPoint::FlushSegmentRecordPrefix,
        DurabilityCrashPoint::SynchronizeSegmentRecordPrefix,
        DurabilityCrashPoint::AppendSegmentSeal,
        DurabilityCrashPoint::FlushSealedSegment,
        DurabilityCrashPoint::SynchronizeSealedSegment,
        DurabilityCrashPoint::LinkSegment,
        DurabilityCrashPoint::SynchronizeSegmentPool,
        DurabilityCrashPoint::RemoveSegmentStage,
        DurabilityCrashPoint::SynchronizeStagingAfterSegment,
    ];
    step(POINTS, point)
}

pub(super) fn catalog_step(
    point: DurabilityCrashPoint,
) -> Result<usize, DurabilityCrashMatrixError> {
    const POINTS: [DurabilityCrashPoint; 8] = [
        DurabilityCrashPoint::CreateCatalogStage,
        DurabilityCrashPoint::WriteCatalog,
        DurabilityCrashPoint::FlushCatalog,
        DurabilityCrashPoint::SynchronizeCatalog,
        DurabilityCrashPoint::LinkCatalog,
        DurabilityCrashPoint::SynchronizeCatalogPool,
        DurabilityCrashPoint::RemoveCatalogStage,
        DurabilityCrashPoint::SynchronizeStagingAfterCatalog,
    ];
    step(POINTS, point)
}

pub(super) fn head_step(point: DurabilityCrashPoint) -> Result<usize, DurabilityCrashMatrixError> {
    const POINTS: [DurabilityCrashPoint; 6] = [
        DurabilityCrashPoint::CreateHeadStage,
        DurabilityCrashPoint::WriteHead,
        DurabilityCrashPoint::FlushHead,
        DurabilityCrashPoint::SynchronizeHead,
        DurabilityCrashPoint::ReplaceHead,
        DurabilityCrashPoint::SynchronizeRootAfterHead,
    ];
    step(POINTS, point)
}

pub(super) fn initialization_step(
    point: DurabilityCrashPoint,
) -> Result<usize, DurabilityCrashMatrixError> {
    const POINTS: [DurabilityCrashPoint; 5] = [
        DurabilityCrashPoint::OpenAndLockWriterFile,
        DurabilityCrashPoint::CreateStagingDirectory,
        DurabilityCrashPoint::CreateSegmentPoolDirectory,
        DurabilityCrashPoint::CreateCatalogPoolDirectory,
        DurabilityCrashPoint::SynchronizeRootAfterInitialization,
    ];
    step(POINTS, point)
}

fn step<const N: usize>(
    points: [DurabilityCrashPoint; N],
    point: DurabilityCrashPoint,
) -> Result<usize, DurabilityCrashMatrixError> {
    points
        .into_iter()
        .position(|candidate| candidate == point)
        .ok_or(DurabilityCrashMatrixError::PointSequenceMismatch { point })
}
