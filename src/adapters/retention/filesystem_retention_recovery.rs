//! This module owns filesystem execution of retention recovery under authority.

use std::io;

use cap_fs_ext::DirExt;
use cap_std::fs::Dir;

use super::filesystem_retention_authority::FilesystemRetentionPublicationAuthority;
use super::filesystem_retention_pool_name as pool_name;
use super::filesystem_retention_recovery_observation::{RetentionRecoveryObservation, StageBytes};
use super::filesystem_retention_stage::{FilesystemRetentionStage, invalid_data};
use super::{
    FilesystemRetentionRecoveryError as Error, RetentionRecoveryReceipt, RetentionRecoveryStorage,
    RetentionStageAssessment, assess_head_stage, assess_manifest_stage, assess_root_stage,
    execute_retention_recovery, plan_retention_recovery,
};
use crate::adapters::filesystem_catalog_artifact::synchronize_directory;
use crate::adapters::filesystem_exact_record::{self as exact_record, EntryIdentity};

/// One retained stage as recovery holds it between steps.
pub(super) enum RecoveredStage {
    /// A complete stage reopened and bound to its identity.
    Complete {
        stage: FilesystemRetentionStage,
        pool_name: String,
        namespace: Option<String>,
    },
    /// A truncated stage identified for discard.
    Truncated {
        identity: EntryIdentity,
        length: u64,
    },
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
                    _ => Ok(truncated(stage)),
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
                    _ => Ok(truncated(stage)),
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
                    _ => Ok(truncated(stage)),
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

fn truncated(stage: &StageBytes) -> RecoveredStage {
    RecoveredStage::Truncated {
        identity: stage.identity,
        length: u64::try_from(stage.bytes.len()).unwrap_or(u64::MAX),
    }
}

impl FilesystemRetentionPublicationAuthority {
    /// Observes, plans, and executes recovery of every fixed retention stage.
    ///
    /// The synchronous call runs under the retained writer lock. A clean store
    /// returns an empty receipt; a truncated pre-effect stage is discarded; a
    /// complete stage is linked and retained as a recovery-protected orphan;
    /// a complete head over linked stages is finalized. Any pending
    /// publication attempt is discarded first. Publication calls this itself
    /// as its first step; callers may also run it explicitly at restart.
    ///
    /// # Errors
    ///
    /// Returns [`FilesystemRetentionRecoveryError`](super::FilesystemRetentionRecoveryError)
    /// at the exact observation failure, planning refusal, or refused step.
    pub fn recover(&mut self) -> Result<RetentionRecoveryReceipt, Error> {
        self.attempt = None;
        self.recovery = None;
        let observation =
            RetentionRecoveryObservation::observe(&self.retention, &self.roots, &self.manifests)
                .map_err(|source| Error::Observe { source })?;
        let plan = plan_retention_recovery(observation.evidence())
            .map_err(|source| Error::Plan { source })?;
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

fn discard_truncated(
    retention: &Dir,
    name: &str,
    slot: &mut Option<RecoveredStage>,
) -> io::Result<()> {
    let Some(RecoveredStage::Truncated { identity, length }) = slot.take() else {
        return Err(invalid_data("recovery step expected a truncated stage"));
    };
    let metadata = retention.symlink_metadata(name)?;
    if !metadata.is_file() || metadata.len() != length || EntryIdentity::from(&metadata) != identity
    {
        return Err(invalid_data(
            "truncated retention stage changed before discard",
        ));
    }
    retention.remove_file(name)?;
    exact_record::require_absent(retention, name)
        .map_err(|_source| invalid_data("discarded retention stage remained visible"))?;
    synchronize_directory(retention)
}

impl RetentionRecoveryStorage for FilesystemRetentionPublicationAuthority {
    fn discard_head_stage(&mut self) -> io::Result<()> {
        let context = self.recovery.as_mut().ok_or_else(no_recovery)?;
        discard_truncated(&self.retention, pool_name::HEAD_STAGE, &mut context.head)
    }

    fn discard_manifest_stage(&mut self) -> io::Result<()> {
        let context = self.recovery.as_mut().ok_or_else(no_recovery)?;
        discard_truncated(
            &self.retention,
            pool_name::MANIFEST_STAGE,
            &mut context.manifest,
        )
    }

    fn discard_root_stage(&mut self) -> io::Result<()> {
        let context = self.recovery.as_mut().ok_or_else(no_recovery)?;
        discard_truncated(&self.retention, pool_name::ROOT_STAGE, &mut context.root)
    }

    fn link_root(&mut self) -> io::Result<()> {
        let context = self.recovery.as_ref().ok_or_else(no_recovery)?;
        let Some(RecoveredStage::Complete {
            stage,
            pool_name: name,
            namespace: Some(namespace),
        }) = context.root.as_ref()
        else {
            return Err(invalid_data("link_root expected a complete root stage"));
        };
        match self.roots.create_dir(namespace) {
            Ok(()) => {}
            Err(source) if source.kind() == io::ErrorKind::AlreadyExists => {}
            Err(source) => return Err(source),
        }
        let directory = self.roots.open_dir_nofollow(namespace)?;
        synchronize_directory(&self.roots)?;
        stage.link(&self.retention, &directory, name)?;
        synchronize_directory(&directory)
    }

    fn link_manifest(&mut self) -> io::Result<()> {
        let context = self.recovery.as_ref().ok_or_else(no_recovery)?;
        let Some(RecoveredStage::Complete {
            stage,
            pool_name: name,
            ..
        }) = context.manifest.as_ref()
        else {
            return Err(invalid_data(
                "link_manifest expected a complete manifest stage",
            ));
        };
        stage.link(&self.retention, &self.manifests, name)?;
        synchronize_directory(&self.manifests)
    }

    fn finalize_head(&mut self) -> io::Result<()> {
        let context = self.recovery.as_mut().ok_or_else(no_recovery)?;
        let (stage, _name, _namespace) = take_complete(&mut context.head)?;
        stage.replace(&self.retention, pool_name::HEAD)?;
        synchronize_directory(&self.retention)
    }

    fn remove_root_stage(&mut self) -> io::Result<()> {
        let context = self.recovery.as_mut().ok_or_else(no_recovery)?;
        let (stage, name, namespace) = take_complete(&mut context.root)?;
        let namespace = namespace.ok_or_else(|| invalid_data("root stage without a namespace"))?;
        let directory = self.roots.open_dir_nofollow(&namespace)?;
        stage.remove(&self.retention, &directory, &name)?;
        synchronize_directory(&self.retention)
    }

    fn remove_manifest_stage(&mut self) -> io::Result<()> {
        let context = self.recovery.as_mut().ok_or_else(no_recovery)?;
        let (stage, name, _namespace) = take_complete(&mut context.manifest)?;
        stage.remove(&self.retention, &self.manifests, &name)?;
        synchronize_directory(&self.retention)
    }
}
