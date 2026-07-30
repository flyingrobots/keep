//! This module binds next-head finalization to pinned filesystem storage.

use std::io;

use super::{
    CatalogRestartError, CatalogRestartPhase, FilesystemRecoveryNextHeadFinalizer,
    RecoveryNextHeadFinalizationReadiness, RecoveryNextHeadFinalizationRequest,
    RecoveryNextHeadFinalizationStorage, RecoveryNextHeadFinalizationStorageError,
    RecoveryNextHeadFinalizationTarget, RecoveryStageNamespacePhase, catalog_restart_loader,
    filesystem_catalog_artifact, filesystem_recovery_next_head_candidate,
};

const HEAD: &str = "HEAD";
const NEXT_HEAD: &str = "head.next";

impl RecoveryNextHeadFinalizationStorage for FilesystemRecoveryNextHeadFinalizer {
    fn verify_current(
        &mut self,
        request: RecoveryNextHeadFinalizationRequest,
    ) -> Result<RecoveryNextHeadFinalizationReadiness, RecoveryNextHeadFinalizationStorageError>
    {
        filesystem_recovery_next_head_candidate::verify_namespaces(
            self,
            RecoveryStageNamespacePhase::BeforeObservation,
        )?;
        let readiness = current_readiness(self, request)?;
        match readiness {
            RecoveryNextHeadFinalizationReadiness::Ready => {
                filesystem_recovery_next_head_candidate::verify(self, request)?;
            }
            RecoveryNextHeadFinalizationReadiness::AlreadyFinalized => {
                require_candidate_absent(self, request)?;
                filesystem_recovery_next_head_candidate::verify_namespaces(
                    self,
                    RecoveryStageNamespacePhase::AfterObservation,
                )?;
            }
        }
        Ok(readiness)
    }

    fn synchronize_candidate(
        &mut self,
        request: RecoveryNextHeadFinalizationRequest,
    ) -> Result<(), RecoveryNextHeadFinalizationStorageError> {
        filesystem_recovery_next_head_candidate::synchronize(self, request)
    }

    fn replace_head(
        &mut self,
        request: RecoveryNextHeadFinalizationRequest,
    ) -> Result<(), RecoveryNextHeadFinalizationStorageError> {
        match self.verify_current(request)? {
            RecoveryNextHeadFinalizationReadiness::Ready => self
                .root()
                .rename(NEXT_HEAD, self.root(), HEAD)
                .map_err(storage),
            RecoveryNextHeadFinalizationReadiness::AlreadyFinalized => {
                Err(RecoveryNextHeadFinalizationStorageError::CurrentMismatch {
                    expected: request.expectation(),
                    observed: Some(request.target()),
                })
            }
        }
    }

    fn synchronize_root(&mut self) -> io::Result<()> {
        filesystem_catalog_artifact::synchronize_directory(self.root())
    }
}

fn current_readiness(
    finalizer: &FilesystemRecoveryNextHeadFinalizer,
    request: RecoveryNextHeadFinalizationRequest,
) -> Result<RecoveryNextHeadFinalizationReadiness, RecoveryNextHeadFinalizationStorageError> {
    match catalog_restart_loader::load_from_directory(finalizer.root(), HEAD, finalizer.policy) {
        Ok(loaded) => {
            let snapshot = loaded.snapshot().map_err(current_view)?;
            let observed = RecoveryNextHeadFinalizationTarget::from_snapshot(&snapshot);
            let expected = request.expectation();
            if expected.current_generation() == Some(observed.generation())
                && expected.current_catalog_digest() == Some(observed.digest())
            {
                Ok(RecoveryNextHeadFinalizationReadiness::Ready)
            } else if observed == request.target() {
                Ok(RecoveryNextHeadFinalizationReadiness::AlreadyFinalized)
            } else {
                Err(RecoveryNextHeadFinalizationStorageError::CurrentMismatch {
                    expected,
                    observed: Some(observed),
                })
            }
        }
        Err(source) if missing_head(&source) => {
            if request.expectation().current_generation().is_none() {
                Ok(RecoveryNextHeadFinalizationReadiness::Ready)
            } else {
                Err(RecoveryNextHeadFinalizationStorageError::CurrentMismatch {
                    expected: request.expectation(),
                    observed: None,
                })
            }
        }
        Err(source) => Err(current_view(source)),
    }
}

fn missing_head(error: &CatalogRestartError) -> bool {
    matches!(
        error,
        CatalogRestartError::Io {
            phase: CatalogRestartPhase::OpenHead,
            source,
        } if source.kind() == io::ErrorKind::NotFound
    )
}

fn current_view(source: CatalogRestartError) -> RecoveryNextHeadFinalizationStorageError {
    RecoveryNextHeadFinalizationStorageError::CurrentView {
        source: Box::new(source),
    }
}

fn require_candidate_absent(
    finalizer: &FilesystemRecoveryNextHeadFinalizer,
    request: RecoveryNextHeadFinalizationRequest,
) -> Result<(), RecoveryNextHeadFinalizationStorageError> {
    match finalizer.root().symlink_metadata(NEXT_HEAD) {
        Err(source) if source.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(source) => Err(storage(source)),
        Ok(_metadata) => Err(
            RecoveryNextHeadFinalizationStorageError::UnexpectedCandidate {
                expected: request.evidence(),
            },
        ),
    }
}

const fn storage(source: io::Error) -> RecoveryNextHeadFinalizationStorageError {
    RecoveryNextHeadFinalizationStorageError::Storage { source }
}
