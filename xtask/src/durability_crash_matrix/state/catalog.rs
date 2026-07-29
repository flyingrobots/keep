//! This module owns Golden File Worldline catalog crash-state construction.

use xtask::{DurabilityCrashCase, DurabilityCrashPoint, DurabilityCrashPosition};

use super::fixture::{CATALOG_POOL_PATH, GoldenFixture, SEGMENT_POOL_PATH};
use super::{DurabilityCrashMatrixError, StoreState};

const STEPS: [DurabilityCrashPoint; 8] = [
    DurabilityCrashPoint::CreateCatalogStage,
    DurabilityCrashPoint::WriteCatalog,
    DurabilityCrashPoint::FlushCatalog,
    DurabilityCrashPoint::SynchronizeCatalog,
    DurabilityCrashPoint::LinkCatalog,
    DurabilityCrashPoint::SynchronizeCatalogPool,
    DurabilityCrashPoint::RemoveCatalogStage,
    DurabilityCrashPoint::SynchronizeStagingAfterCatalog,
];

pub(super) fn prepare(
    state: &mut StoreState,
    case: DurabilityCrashCase,
) -> Result<(), DurabilityCrashMatrixError> {
    state.initialize()?;
    state.write_immutable(SEGMENT_POOL_PATH, &GoldenFixture::segment()?)?;
    let fixture = GoldenFixture::catalog()?;
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
        DurabilityCrashPoint::CreateCatalogStage => state.create_stage("staging/current.cat"),
        DurabilityCrashPoint::WriteCatalog => {
            let end = if position == DurabilityCrashPosition::During {
                176
            } else {
                352
            };
            state.write_range(fixture, 0..end)
        }
        DurabilityCrashPoint::FlushCatalog => after(position, || state.flush()),
        DurabilityCrashPoint::SynchronizeCatalog => after(position, || state.synchronize_file()),
        DurabilityCrashPoint::LinkCatalog => state.link("staging/current.cat", CATALOG_POOL_PATH),
        DurabilityCrashPoint::SynchronizeCatalogPool => {
            after(position, || state.synchronize_directory("catalogs"))
        }
        DurabilityCrashPoint::RemoveCatalogStage => state.remove("staging/current.cat"),
        DurabilityCrashPoint::SynchronizeStagingAfterCatalog => {
            after(position, || state.synchronize_directory("staging"))
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
