//! This module owns explicit-discard crash-state construction.

use xtask::{DurabilityCrashCase, DurabilityCrashPoint, DurabilityCrashPosition};

use super::fixture::GoldenFixture;
use super::{DurabilityCrashMatrixError, StoreState};

pub(super) fn prepare(
    state: &mut StoreState,
    case: DurabilityCrashCase,
) -> Result<(), DurabilityCrashMatrixError> {
    state.initialize()?;
    match case.point() {
        DurabilityCrashPoint::RemoveRecoveryStage
        | DurabilityCrashPoint::SynchronizeStagingAfterRecovery => {
            prepare_segment_discard(state, case)
        }
        DurabilityCrashPoint::RemoveRecoveryHead
        | DurabilityCrashPoint::SynchronizeRootAfterRecovery => prepare_head_discard(state, case),
        point => Err(DurabilityCrashMatrixError::PointSequenceMismatch { point }),
    }
}

fn prepare_segment_discard(
    state: &mut StoreState,
    case: DurabilityCrashCase,
) -> Result<(), DurabilityCrashMatrixError> {
    state.create_stage("staging/current.seg")?;
    state.write_range(&GoldenFixture::segment()?, 0..32)?;
    if case.point() == DurabilityCrashPoint::RemoveRecoveryStage {
        if case.position() != DurabilityCrashPosition::Before {
            state.remove("staging/current.seg")?;
        }
        return Ok(());
    }
    state.remove("staging/current.seg")?;
    if case.position() == DurabilityCrashPosition::After {
        state.synchronize_directory("staging")?;
    }
    Ok(())
}

fn prepare_head_discard(
    state: &mut StoreState,
    case: DurabilityCrashCase,
) -> Result<(), DurabilityCrashMatrixError> {
    state.create_stage("head.next")?;
    state.write_range(&GoldenFixture::head()?, 0..64)?;
    if case.point() == DurabilityCrashPoint::RemoveRecoveryHead {
        if case.position() != DurabilityCrashPosition::Before {
            state.remove("head.next")?;
        }
        return Ok(());
    }
    state.remove("head.next")?;
    if case.position() == DurabilityCrashPosition::After {
        state.synchronize_directory(".")?;
    }
    Ok(())
}
