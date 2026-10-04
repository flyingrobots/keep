//! This module owns retained stage handles and exact bytes across recovery planning.

use std::io;

use super::recovery::{FilesystemCompactionRecoveryError as Error, refused};
use crate::adapters::filesystem_recovery_stage::{self, ObservedRecoveryStage};
use crate::adapters::{
    FilesystemRecoveryStageDiscarder, FilesystemRecoveryStageError, RecoveryStage,
    RecoveryStageNamespacePhase, admit_recovery_stage_bytes,
};

pub(super) struct ObservedStage {
    stage: RecoveryStage,
    opened: ObservedRecoveryStage,
    bytes: Box<[u8]>,
}

impl ObservedStage {
    pub(super) const fn stage(&self) -> RecoveryStage {
        self.stage
    }

    pub(super) fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Rechecks the original opened file, its bytes and its pinned namespace.
    /// This guard is not atomic with unlink against unsupported raw mutation.
    pub(super) fn verify(
        &mut self,
        discarder: &FilesystemRecoveryStageDiscarder,
    ) -> Result<(), Error> {
        let inventory = &discarder.inventory;
        inventory
            .verify_stage_namespaces(self.stage, RecoveryStageNamespacePhase::BeforeObservation)
            .map_err(|source| refused("verify stage namespace", source))?;
        let directory = inventory.stage_directory(self.stage);
        self.opened
            .verify(directory, self.stage.file_name(), self.stage)
            .map_err(|source| refused("verify stage identity", source))?;
        let bytes = self
            .opened
            .materialize_and_position(self.stage)
            .map_err(|source| refused("verify stage bytes", source))?;
        let _admitted = admit_recovery_stage_bytes(self.stage, self.opened.evidence(), &bytes)
            .map_err(|source| refused("verify stage bytes", source))?;
        if bytes != self.bytes {
            return Err(refused(
                "verify stage bytes",
                io::Error::other("the stage changed after assessment"),
            ));
        }
        self.opened
            .verify(directory, self.stage.file_name(), self.stage)
            .map_err(|source| refused("verify stage identity", source))?;
        inventory
            .verify_stage_namespaces(self.stage, RecoveryStageNamespacePhase::AfterObservation)
            .map_err(|source| refused("verify stage namespace", source))
    }
}

pub(super) fn read_stage(
    discarder: &FilesystemRecoveryStageDiscarder,
    stage: RecoveryStage,
) -> Result<Option<ObservedStage>, Error> {
    let inventory = &discarder.inventory;
    inventory
        .verify_stage_namespaces(stage, RecoveryStageNamespacePhase::BeforeObservation)
        .map_err(|source| refused("verify stage namespace", source))?;
    let directory = inventory.stage_directory(stage);
    let observation = filesystem_recovery_stage::observe(directory, stage);
    inventory
        .verify_stage_namespaces(stage, RecoveryStageNamespacePhase::AfterObservation)
        .map_err(|source| refused("verify stage namespace", source))?;
    let mut opened = match observation {
        Ok(observed) => observed,
        Err(FilesystemRecoveryStageError::Open { source, .. })
            if source.kind() == io::ErrorKind::NotFound =>
        {
            return Ok(None);
        }
        Err(source) => return Err(refused("observe stage", source)),
    };
    let bytes = opened
        .materialize_and_position(stage)
        .map_err(|source| refused("read stage", source))?;
    let _admitted = admit_recovery_stage_bytes(stage, opened.evidence(), &bytes)
        .map_err(|source| refused("verify stage bytes", source))?;
    opened
        .verify(directory, stage.file_name(), stage)
        .map_err(|source| refused("verify stage entry", source))?;
    inventory
        .verify_stage_namespaces(stage, RecoveryStageNamespacePhase::AfterObservation)
        .map_err(|source| refused("verify stage namespace", source))?;
    Ok(Some(ObservedStage {
        stage,
        opened,
        bytes,
    }))
}
