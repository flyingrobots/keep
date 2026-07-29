//! This module owns crash-safe initialization state construction.

use xtask::{DurabilityCrashCase, DurabilityCrashPoint, DurabilityCrashPosition};

use super::{DurabilityCrashMatrixError, StoreState};

const STEPS: [DurabilityCrashPoint; 5] = [
    DurabilityCrashPoint::OpenAndLockWriterFile,
    DurabilityCrashPoint::CreateStagingDirectory,
    DurabilityCrashPoint::CreateSegmentPoolDirectory,
    DurabilityCrashPoint::CreateCatalogPoolDirectory,
    DurabilityCrashPoint::SynchronizeRootAfterInitialization,
];

pub(super) fn prepare(
    state: &mut StoreState,
    case: DurabilityCrashCase,
) -> Result<(), DurabilityCrashMatrixError> {
    for step in STEPS {
        let position = if step == case.point() {
            case.position()
        } else {
            DurabilityCrashPosition::After
        };
        apply(state, step, position)?;
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
    point: DurabilityCrashPoint,
    position: DurabilityCrashPosition,
) -> Result<(), DurabilityCrashMatrixError> {
    if position == DurabilityCrashPosition::Before {
        return Ok(());
    }
    match point {
        DurabilityCrashPoint::OpenAndLockWriterFile => {
            state.create_writer_lock()?;
            state.acquire_writer_lock()
        }
        DurabilityCrashPoint::CreateStagingDirectory => state.create_directory("staging"),
        DurabilityCrashPoint::CreateSegmentPoolDirectory => state.create_directory("segments"),
        DurabilityCrashPoint::CreateCatalogPoolDirectory => state.create_directory("catalogs"),
        DurabilityCrashPoint::SynchronizeRootAfterInitialization => {
            if position == DurabilityCrashPosition::After {
                state.synchronize_directory(".")
            } else {
                Ok(())
            }
        }
        _ => Err(DurabilityCrashMatrixError::PointSequenceMismatch { point }),
    }
}
