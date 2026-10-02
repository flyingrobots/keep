//! This module owns filesystem execution of retention recovery under authority.

use super::RetentionStorageError;
use std::io;

use cap_fs_ext::DirExt;
use cap_std::fs::Dir;

use super::filesystem_retention_authority::FilesystemRetentionPublicationAuthority;
use super::filesystem_retention_pool_name as pool_name;
use super::filesystem_retention_recovery_observation::RetentionRecoveryObservation;
use super::filesystem_retention_stage::{FilesystemRetentionStage, invalid_data};
use super::filesystem_retention_storage::require_pinned_directories;
use super::{
    FilesystemRetentionRecoveryError as Error, RetentionRecoveryReceipt, RetentionRecoveryStorage,
    RetentionStageAssessment, assess_head_stage, assess_manifest_stage, assess_root_stage,
    execute_retention_recovery, plan_retention_recovery,
};
use crate::adapters::CatalogRestartPolicy;
use crate::adapters::filesystem_catalog_artifact::synchronize_directory;

#[cfg(test)]
#[path = "filesystem_retention_recovery_storage_error_tests.rs"]
mod storage_error_tests;

/// One retained stage as recovery holds it between steps.
pub(super) enum RecoveredStage {
    /// A complete stage reopened and bound to its identity.
    Complete {
        stage: FilesystemRetentionStage,
        pool_name: String,
        namespace: Option<String>,
    },
    /// Incomplete evidence cannot authorize a filesystem mutation.
    Incomplete,
}

/// The retained stages one recovery run operates on.
pub(super) struct RetentionRecoveryContext {
    root: Option<RecoveredStage>,
    manifest: Option<RecoveredStage>,
    head: Option<RecoveredStage>,
}

impl RetentionRecoveryContext {
    fn reopen(retention: &Dir, observation: &RetentionRecoveryObservation) -> io::Result<Self> {
        let root = observation
            .root()
            .map(|stage| -> io::Result<RecoveredStage> {
                match assess_root_stage(Some(&stage.bytes)) {
                    RetentionStageAssessment::Complete(admitted) => Ok(RecoveredStage::Complete {
                        stage: FilesystemRetentionStage::reopen(
                            retention,
                            pool_name::ROOT_STAGE,
                            &stage.bytes,
                        )?,
                        pool_name: pool_name::root(admitted.root().generation(), admitted.digest()),
                        namespace: Some(pool_name::namespace(admitted.root().namespace().digest())),
                    }),
                    _ => Ok(RecoveredStage::Incomplete),
                }
            })
            .transpose()?;
        let manifest = observation
            .manifest()
            .map(|stage| -> io::Result<RecoveredStage> {
                match assess_manifest_stage(Some(&stage.bytes)) {
                    RetentionStageAssessment::Complete(admitted) => Ok(RecoveredStage::Complete {
                        stage: FilesystemRetentionStage::reopen(
                            retention,
                            pool_name::MANIFEST_STAGE,
                            &stage.bytes,
                        )?,
                        pool_name: pool_name::manifest(
                            admitted.manifest().generation(),
                            admitted.digest(),
                        ),
                        namespace: None,
                    }),
                    _ => Ok(RecoveredStage::Incomplete),
                }
            })
            .transpose()?;
        let head = observation
            .head()
            .map(|stage| -> io::Result<RecoveredStage> {
                match assess_head_stage(Some(&stage.bytes)) {
                    RetentionStageAssessment::Complete(_) => Ok(RecoveredStage::Complete {
                        stage: FilesystemRetentionStage::reopen(
                            retention,
                            pool_name::HEAD_STAGE,
                            &stage.bytes,
                        )?,
                        pool_name: pool_name::HEAD.to_owned(),
                        namespace: None,
                    }),
                    _ => Ok(RecoveredStage::Incomplete),
                }
            })
            .transpose()?;
        Ok(Self {
            root,
            manifest,
            head,
        })
    }
}

impl FilesystemRetentionPublicationAuthority {
    /// Recovers fixed retention stages with an explicit catalog loading policy.
    ///
    /// The synchronous call runs under the retained writer lock. A clean store
    /// returns an empty receipt; an incomplete stage requires disposition; a
    /// complete stage is linked and retained as a recovery-protected orphan;
    /// a complete head over linked stages is finalized. Any pending
    /// publication attempt is discarded first. Publication calls this itself
    /// as its first step; callers may also run it explicitly at restart.
    /// Both entry points verify that protocol names still identify the pinned
    /// directories before observing stages or executing recovery effects.
    /// A complete root requires loading every head-selected catalog segment
    /// into bounded retained memory, then replaying its anchors and closure
    /// limits before any recovery effect. The supplied policy bounds aggregate
    /// segment bytes and per-segment record admission; catalog bytes also have
    /// their independent protocol bound. Clean and incomplete-root recovery
    /// does not materialize the catalog. This call may allocate and block on I/O.
    ///
    /// # Errors
    ///
    /// Returns [`FilesystemRetentionRecoveryError`](super::FilesystemRetentionRecoveryError)
    /// at the exact observation failure, planning refusal, or refused step.
    pub fn recover_with_catalog_policy(
        &mut self,
        policy: CatalogRestartPolicy,
    ) -> Result<RetentionRecoveryReceipt, Error> {
        require_pinned_directories(&self.root, &self.retention, &self.roots, &self.manifests)
            .map_err(|source| Error::Observe { source })?;
        self.attempt = None;
        self.recovery = None;
        let _census = super::filesystem_retention_namespace::admit_recovery(
            &self.retention,
            &self.roots,
            &self.manifests,
        )
        .map_err(|source| Error::Observe { source })?;
        let observation =
            RetentionRecoveryObservation::observe(&self.retention, &self.roots, &self.manifests)
                .map_err(|source| Error::Observe { source })?;
        let plan = plan_retention_recovery(observation.evidence())
            .map_err(|source| Error::Plan { source })?;
        super::filesystem_retention_recovery_roots::admit(&self.roots, &observation.evidence())
            .map_err(|source| Error::Observe { source })?;
        super::filesystem_retention_closure_admission::admit_recovery(
            &self.root,
            &observation.evidence(),
            policy,
        )
        .map_err(|source| Error::Observe { source })?;
        self.recovery = Some(
            RetentionRecoveryContext::reopen(&self.retention, &observation)
                .map_err(|source| Error::Observe { source })?,
        );
        let result =
            execute_retention_recovery(self, &plan).map_err(|source| Error::Execute { source });
        self.recovery = None;
        result
    }
}

fn no_recovery() -> io::Error {
    invalid_data("no retention recovery is in progress")
}

fn take_complete(
    slot: &mut Option<RecoveredStage>,
) -> io::Result<(FilesystemRetentionStage, String, Option<String>)> {
    match slot.take() {
        Some(RecoveredStage::Complete {
            stage,
            pool_name,
            namespace,
        }) => Ok((stage, pool_name, namespace)),
        Some(other) => {
            *slot = Some(other);
            Err(invalid_data("recovery step expected a complete stage"))
        }
        None => Err(invalid_data("recovery step expected a retained stage")),
    }
}

const fn disposition_required() -> Result<(), RetentionStorageError> {
    Err(RetentionStorageError::Refused {
        source: super::RetentionRecordRefusal::IncompleteDispositionRequired,
    })
}

impl RetentionRecoveryStorage for FilesystemRetentionPublicationAuthority {
    fn discard_head_stage(&mut self) -> Result<(), RetentionStorageError> {
        disposition_required()
    }

    fn discard_manifest_stage(&mut self) -> Result<(), RetentionStorageError> {
        disposition_required()
    }

    fn discard_root_stage(&mut self) -> Result<(), RetentionStorageError> {
        disposition_required()
    }

    fn link_root(&mut self) -> Result<(), RetentionStorageError> {
        let context = self.recovery.as_ref().ok_or_else(no_recovery)?;
        let Some(RecoveredStage::Complete {
            stage,
            pool_name: name,
            namespace: Some(namespace),
        }) = context.root.as_ref()
        else {
            return Err(invalid_data("link_root expected a complete root stage").into());
        };
        stage.synchronize(&self.retention)?;
        match self.roots.create_dir(namespace) {
            Ok(()) => {}
            Err(source) if source.kind() == io::ErrorKind::AlreadyExists => {}
            Err(source) => return Err(source.into()),
        }
        let directory = self.roots.open_dir_nofollow(namespace)?;
        synchronize_directory(&self.roots)?;
        stage.link(&self.retention, &directory, name)?;
        synchronize_directory(&directory).map_err(Into::into)
    }

    fn link_manifest(&mut self) -> Result<(), RetentionStorageError> {
        let context = self.recovery.as_ref().ok_or_else(no_recovery)?;
        let Some(RecoveredStage::Complete {
            stage,
            pool_name: name,
            ..
        }) = context.manifest.as_ref()
        else {
            return Err(invalid_data("link_manifest expected a complete manifest stage").into());
        };
        stage.synchronize(&self.retention)?;
        stage.link(&self.retention, &self.manifests, name)?;
        synchronize_directory(&self.manifests).map_err(Into::into)
    }

    fn finalize_head(&mut self) -> Result<(), RetentionStorageError> {
        let context = self.recovery.as_mut().ok_or_else(no_recovery)?;
        let (stage, _name, _namespace) = take_complete(&mut context.head)?;
        stage.synchronize(&self.retention)?;
        stage.replace(&self.retention, pool_name::HEAD)?;
        self.synchronize_recovery_directory(
            &self.retention,
            super::RetentionStorageBoundary::RetentionSynchronization,
        )
    }

    fn remove_root_stage(&mut self) -> Result<(), RetentionStorageError> {
        let context = self.recovery.as_mut().ok_or_else(no_recovery)?;
        let (stage, name, namespace) = take_complete(&mut context.root)?;
        let namespace = namespace.ok_or_else(|| invalid_data("root stage without a namespace"))?;
        let directory = self.roots.open_dir_nofollow(&namespace)?;
        stage.remove(&self.retention, &directory, &name)?;
        self.synchronize_recovery_directory(
            &self.retention,
            super::RetentionStorageBoundary::RetentionSynchronization,
        )
    }

    fn remove_manifest_stage(&mut self) -> Result<(), RetentionStorageError> {
        let context = self.recovery.as_mut().ok_or_else(no_recovery)?;
        let (stage, name, _namespace) = take_complete(&mut context.manifest)?;
        stage.remove(&self.retention, &self.manifests, &name)?;
        self.synchronize_recovery_directory(
            &self.retention,
            super::RetentionStorageBoundary::RetentionSynchronization,
        )
    }
}

impl FilesystemRetentionPublicationAuthority {
    fn synchronize_recovery_directory(
        &self,
        directory: &Dir,
        boundary: super::RetentionStorageBoundary,
    ) -> Result<(), RetentionStorageError> {
        #[cfg(test)]
        if self.recovery_sync_failure == Some(boundary) {
            return Err(io::Error::from_raw_os_error(rustix::io::Errno::IO.raw_os_error()).into());
        }
        let _ = boundary;
        synchronize_directory(directory).map_err(Into::into)
    }
}
