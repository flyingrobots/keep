//! This module owns Golden File Worldline segment crash-state construction.

use xtask::{DurabilityCrashCase, DurabilityCrashPoint, DurabilityCrashPosition};

use super::fixture::{GoldenFixture, SEGMENT_POOL_PATH};
use super::{DurabilityCrashMatrixError, StoreState};

const STEPS: [DurabilityCrashPoint; 12] = [
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

pub(super) fn prepare(
    state: &mut StoreState,
    case: DurabilityCrashCase,
) -> Result<(), DurabilityCrashMatrixError> {
    state.initialize()?;
    let fixture = GoldenFixture::segment()?;
    for step in STEPS {
        let position = if step == case.point() {
            case.position()
        } else {
            DurabilityCrashPosition::After
        };
        apply(state, &fixture, step, position)?;
        if step == case.point() {
            return Ok(());
        }
    }
    Err(DurabilityCrashMatrixError::PointSequenceMismatch {
        point: case.point(),
    })
}

fn apply(
    state: &mut StoreState,
    fixture: &GoldenFixture,
    point: DurabilityCrashPoint,
    position: DurabilityCrashPosition,
) -> Result<(), DurabilityCrashMatrixError> {
    if position == DurabilityCrashPosition::Before {
        return Ok(());
    }
    match point {
        DurabilityCrashPoint::CreateSegmentStage => state.create_stage("staging/current.seg"),
        DurabilityCrashPoint::WriteSegmentHeader => {
            let end = interrupted_end(position, 32, 64);
            state.write_range(fixture, 0..end)
        }
        DurabilityCrashPoint::AppendSegmentRecord => {
            let end = interrupted_end(position, 136, 209);
            state.write_range(fixture, 64..end)
        }
        DurabilityCrashPoint::FlushSegmentRecordPrefix
        | DurabilityCrashPoint::FlushSealedSegment => after(position, || state.flush()),
        DurabilityCrashPoint::SynchronizeSegmentRecordPrefix
        | DurabilityCrashPoint::SynchronizeSealedSegment => {
            after(position, || state.synchronize_file())
        }
        DurabilityCrashPoint::AppendSegmentSeal => {
            let end = interrupted_end(position, 273, 337);
            state.write_range(fixture, 209..end)
        }
        DurabilityCrashPoint::LinkSegment => state.link("staging/current.seg", SEGMENT_POOL_PATH),
        DurabilityCrashPoint::SynchronizeSegmentPool => {
            after(position, || state.synchronize_directory("segments"))
        }
        DurabilityCrashPoint::RemoveSegmentStage => state.remove("staging/current.seg"),
        DurabilityCrashPoint::SynchronizeStagingAfterSegment => {
            after(position, || state.synchronize_directory("staging"))
        }
        _ => Err(DurabilityCrashMatrixError::PointSequenceMismatch { point }),
    }
}

fn interrupted_end(position: DurabilityCrashPosition, during: usize, after: usize) -> usize {
    if position == DurabilityCrashPosition::During {
        during
    } else {
        after
    }
}

fn after(
    position: DurabilityCrashPosition,
    operation: impl FnOnce() -> Result<(), DurabilityCrashMatrixError>,
) -> Result<(), DurabilityCrashMatrixError> {
    if position == DurabilityCrashPosition::After {
        operation()
    } else {
        Ok(())
    }
}
