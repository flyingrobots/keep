//! This module owns filesystem execution of reusable segment continuation.

use std::io;

use super::{
    FilesystemRecoverySegmentResumer, FilesystemRecoverySegmentStage, FilesystemRecoveryStageError,
    OpenedReusableSegment, RecoverySegmentResumeRequest, RecoverySegmentResumeStorage,
    RecoverySegmentResumeStorageError, RecoveryStage, RecoveryStageNamespacePhase,
    filesystem_recovery_stage,
};

impl RecoverySegmentResumeStorage for FilesystemRecoverySegmentResumer {
    type Stage = FilesystemRecoverySegmentStage;

    fn open_reusable(
        self,
        request: RecoverySegmentResumeRequest,
    ) -> Result<OpenedReusableSegment<Self::Stage>, RecoverySegmentResumeStorageError> {
        #[cfg(test)]
        let result = {
            let mut storage = self;
            if let Some(before_handoff) = storage.before_handoff.take() {
                return open_with(storage, request, before_handoff);
            }
            open_with(storage, request, || {})
        };
        #[cfg(not(test))]
        let result = open_with(self, request, || {});
        result
    }
}

fn open_with<F>(
    resumer: FilesystemRecoverySegmentResumer,
    request: RecoverySegmentResumeRequest,
    before_handoff: F,
) -> Result<OpenedReusableSegment<FilesystemRecoverySegmentStage>, RecoverySegmentResumeStorageError>
where
    F: FnOnce(),
{
    let inventory = &resumer.discarder.inventory;
    inventory
        .verify_stage_namespaces(
            RecoveryStage::Segment,
            RecoveryStageNamespacePhase::BeforeObservation,
        )
        .map_err(stage_error)?;
    let directory = inventory.stage_directory(RecoveryStage::Segment);
    let mut observed =
        match filesystem_recovery_stage::observe_writable_segment_with(directory, || {}) {
            Ok(observed) => observed,
            Err(FilesystemRecoveryStageError::Open { source, .. })
                if source.kind() == io::ErrorKind::NotFound =>
            {
                return Err(RecoverySegmentResumeStorageError::Missing { request });
            }
            Err(source) => return Err(stage_error(source)),
        };
    let actual = observed.evidence();
    if actual != request.evidence() {
        return Err(RecoverySegmentResumeStorageError::EvidenceMismatch {
            expected: request.evidence(),
            observed: actual,
        });
    }
    let encoded = observed
        .materialize_and_position(RecoveryStage::Segment)
        .map_err(stage_error)?;
    before_handoff();
    inventory
        .verify_stage_namespaces(
            RecoveryStage::Segment,
            RecoveryStageNamespacePhase::AfterObservation,
        )
        .map_err(stage_error)?;
    observed
        .verify(
            directory,
            RecoveryStage::Segment.file_name(),
            RecoveryStage::Segment,
        )
        .map_err(stage_error)?;
    let final_evidence = observed
        .refingerprint_and_position(RecoveryStage::Segment)
        .map_err(stage_error)?;
    if final_evidence != request.evidence() {
        return Err(RecoverySegmentResumeStorageError::EvidenceMismatch {
            expected: request.evidence(),
            observed: final_evidence,
        });
    }
    observed
        .verify(
            directory,
            RecoveryStage::Segment.file_name(),
            RecoveryStage::Segment,
        )
        .map_err(stage_error)?;
    let file = observed.into_file();
    let stage = FilesystemRecoverySegmentStage::new(file, resumer.discarder);
    Ok(OpenedReusableSegment::new(stage, encoded))
}

fn stage_error(source: FilesystemRecoveryStageError) -> RecoverySegmentResumeStorageError {
    RecoverySegmentResumeStorageError::storage(io::Error::other(source))
}
