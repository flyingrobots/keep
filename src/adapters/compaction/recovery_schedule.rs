//! This module owns deterministic fault scheduling around recovery storage I/O.
//! Production supplies a no-op callback; the original storage capabilities own
//! identity, exact-byte verification, authority and actual filesystem effects.

use std::io;

use super::CompactionRecoveryBoundary as Boundary;
use crate::adapters::{
    FilesystemRecoveryNextHeadFinalizer, FilesystemRecoveryStageDiscarder,
    RecoveryNextHeadFinalizationReadiness, RecoveryNextHeadFinalizationRequest,
    RecoveryNextHeadFinalizationStorage, RecoveryNextHeadFinalizationStorageError, RecoveryStage,
    RecoveryStageDiscardOutcome, RecoveryStageDiscardStorage, RecoveryStageDiscardStorageError,
    RecoveryStageEvidence, RecoveryStageParent,
};

pub(super) type Schedule<'a> = dyn FnMut(RecoveryStage, Boundary) -> io::Result<()> + 'a;

pub(super) struct ScheduledDiscard<'a, 'env> {
    pub(super) inner: &'a mut FilesystemRecoveryStageDiscarder,
    pub(super) stage: RecoveryStage,
    pub(super) schedule: &'a mut Schedule<'env>,
}

impl RecoveryStageDiscardStorage for ScheduledDiscard<'_, '_> {
    fn remove_if_matching(
        &mut self,
        evidence: RecoveryStageEvidence,
    ) -> Result<RecoveryStageDiscardOutcome, RecoveryStageDiscardStorageError> {
        self.inner.remove_if_matching(evidence)
    }

    fn synchronize_parent(&mut self, parent: RecoveryStageParent) -> io::Result<()> {
        (self.schedule)(self.stage, Boundary::SynchronizeParent)?;
        self.inner.synchronize_parent(parent)
    }
}

pub(super) struct ScheduledFinalization<'a, 'env> {
    pub(super) inner: FilesystemRecoveryNextHeadFinalizer,
    pub(super) schedule: &'a mut Schedule<'env>,
}

impl RecoveryNextHeadFinalizationStorage for ScheduledFinalization<'_, '_> {
    fn verify_current(
        &mut self,
        request: RecoveryNextHeadFinalizationRequest,
    ) -> Result<RecoveryNextHeadFinalizationReadiness, RecoveryNextHeadFinalizationStorageError>
    {
        self.inner.verify_current(request)
    }

    fn synchronize_candidate(
        &mut self,
        request: RecoveryNextHeadFinalizationRequest,
    ) -> Result<(), RecoveryNextHeadFinalizationStorageError> {
        self.inner.synchronize_candidate(request)
    }

    fn replace_head(
        &mut self,
        request: RecoveryNextHeadFinalizationRequest,
    ) -> Result<(), RecoveryNextHeadFinalizationStorageError> {
        self.inner.replace_head(request)
    }

    fn synchronize_root(&mut self) -> io::Result<()> {
        (self.schedule)(RecoveryStage::NextHead, Boundary::SynchronizeRoot)?;
        self.inner.synchronize_root()
    }
}
