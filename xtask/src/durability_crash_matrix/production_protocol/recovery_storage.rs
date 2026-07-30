//! This module owns crash injection around production recovery discard.

use std::io;

use keep::{
    FilesystemRecoveryStageDiscarder, RecoveryStage, RecoveryStageDiscardOutcome,
    RecoveryStageDiscardStorage, RecoveryStageDiscardStorageError, RecoveryStageEvidence,
    RecoveryStageParent,
};
use xtask::DurabilityCrashPoint;

use super::control::{CrashControl, DuringTiming};

pub(super) struct CrashRecoveryStorage<'control> {
    inner: FilesystemRecoveryStageDiscarder,
    control: &'control mut CrashControl,
}

impl<'control> CrashRecoveryStorage<'control> {
    pub(super) const fn new(
        inner: FilesystemRecoveryStageDiscarder,
        control: &'control mut CrashControl,
    ) -> Self {
        Self { inner, control }
    }
}

impl RecoveryStageDiscardStorage for CrashRecoveryStorage<'_> {
    fn remove_if_matching(
        &mut self,
        expected: RecoveryStageEvidence,
    ) -> Result<RecoveryStageDiscardOutcome, RecoveryStageDiscardStorageError> {
        let point = match expected.stage() {
            RecoveryStage::NextHead => DurabilityCrashPoint::RemoveRecoveryHead,
            RecoveryStage::Segment | RecoveryStage::Catalog => {
                DurabilityCrashPoint::RemoveRecoveryStage
            }
        };
        self.control
            .before(point, DuringTiming::After)
            .map_err(storage_error)?;
        let outcome = self.inner.remove_if_matching(expected)?;
        self.control
            .after(point, DuringTiming::After)
            .map_err(storage_error)?;
        Ok(outcome)
    }

    fn synchronize_parent(&mut self, parent: RecoveryStageParent) -> io::Result<()> {
        let point = match parent {
            RecoveryStageParent::Staging => DurabilityCrashPoint::SynchronizeStagingAfterRecovery,
            RecoveryStageParent::Root => DurabilityCrashPoint::SynchronizeRootAfterRecovery,
        };
        self.control.before(point, DuringTiming::Before)?;
        self.inner.synchronize_parent(parent)?;
        self.control.after(point, DuringTiming::Before)
    }
}

const fn storage_error(source: io::Error) -> RecoveryStageDiscardStorageError {
    RecoveryStageDiscardStorageError::Storage { source }
}
