//! This module owns Golden File Worldline publication-head crash states.

use xtask::{DurabilityCrashCase, DurabilityCrashPoint, DurabilityCrashPosition};

use super::fixture::{CATALOG_POOL_PATH, GoldenFixture, SEGMENT_POOL_PATH};
use super::{DurabilityCrashMatrixError, StoreState};

const STEPS: [DurabilityCrashPoint; 6] = [
    DurabilityCrashPoint::CreateHeadStage,
    DurabilityCrashPoint::WriteHead,
    DurabilityCrashPoint::FlushHead,
    DurabilityCrashPoint::SynchronizeHead,
    DurabilityCrashPoint::ReplaceHead,
    DurabilityCrashPoint::SynchronizeRootAfterHead,
];

pub(super) fn prepare(
    state: &mut StoreState,
    case: DurabilityCrashCase,
) -> Result<(), DurabilityCrashMatrixError> {
    state.initialize()?;
    state.write_immutable(SEGMENT_POOL_PATH, &GoldenFixture::segment()?)?;
    state.write_immutable(CATALOG_POOL_PATH, &GoldenFixture::catalog()?)?;
    let fixture = GoldenFixture::head()?;
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
        DurabilityCrashPoint::CreateHeadStage => state.create_stage("head.next"),
        DurabilityCrashPoint::WriteHead => {
            let end = if position == DurabilityCrashPosition::During {
                64
            } else {
                128
            };
            state.write_range(fixture, 0..end)
        }
        DurabilityCrashPoint::FlushHead => after(position, || state.flush()),
        DurabilityCrashPoint::SynchronizeHead => after(position, || state.synchronize_file()),
        DurabilityCrashPoint::ReplaceHead => state.rename("head.next", "HEAD"),
        DurabilityCrashPoint::SynchronizeRootAfterHead => {
            after(position, || state.synchronize_directory("."))
        }
        _ => Err(DurabilityCrashMatrixError::PointSequenceMismatch { point }),
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
