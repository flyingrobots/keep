//! This module owns exact writer-locked filesystem retention authority.

use std::io;

use cap_fs_ext::DirExt;
use cap_std::fs::Dir;

use super::filesystem_retention_authority_error::{
    FilesystemRetentionAuthorityError as Error, RetentionAuthorityDirectory as Directory,
};
use super::filesystem_retention_current::{self, ObservedRetentionState};
use super::filesystem_retention_pool_name as pool_name;
use super::filesystem_retention_stage::{FilesystemRetentionStage, invalid_data};
use crate::adapters::{FilesystemVersionTwoAdmission, FilesystemWriterLock};

/// Exclusive authority to publish retention transitions on one pinned root.
///
/// The authority retains the admitted writer lock and pinned `retention`,
/// `retention/roots`, and `retention/manifests` capabilities for its entire
/// lifetime. When passed to
/// [`execute_retention_publication`](crate::execute_retention_publication) its
/// [`RetentionPublicationStorage`](super::RetentionPublicationStorage)
/// implementation executes only the forward publication protocol from an
/// exactly admitted current state. It retains opened stage handles through
/// final verification, performs synchronous capability-relative I/O, and uses
/// neither a network nor an asynchronous runtime. Reopening a retained stage
/// prefix remains a separate recovery boundary.
#[must_use]
pub struct FilesystemRetentionPublicationAuthority {
    pub(super) root: Dir,
    pub(super) retention: Dir,
    pub(super) roots: Dir,
    pub(super) manifests: Dir,
    pub(super) namespace: Option<Dir>,
    pub(super) liveness_generation: Option<crate::LivenessGeneration>,
    pub(super) retained_root: Option<String>,
    pub(super) retained_manifest: Option<String>,
    pub(super) root_stage: Option<FilesystemRetentionStage>,
    pub(super) manifest_stage: Option<FilesystemRetentionStage>,
    pub(super) head_stage: Option<FilesystemRetentionStage>,
    _lock: FilesystemWriterLock,
}

impl FilesystemRetentionPublicationAuthority {
    /// Pins one admitted version-two root for retention publication.
    ///
    /// Only [`FilesystemVersionTwoAdmission`] is accepted, so version-one
    /// writer authority can never reach retention publication.
    ///
    /// This synchronous constructor opens pinned directory capabilities but
    /// materializes no record bodies and performs no protocol mutation.
    ///
    /// # Errors
    ///
    /// Returns [`FilesystemRetentionAuthorityError`](super::FilesystemRetentionAuthorityError)
    /// when the root capability cannot be cloned or the retention namespace and
    /// either immutable pool cannot be pinned without following links.
    pub fn open(admission: FilesystemVersionTwoAdmission) -> Result<Self, Error> {
        let lock = admission.into_lock();
        let root = lock.clone_directory().map_err(|source| Error::Directory {
            directory: Directory::Root,
            source,
        })?;
        let retention = open_directory(&root, pool_name::RETENTION, Directory::Retention)?;
        let roots = open_directory(&retention, pool_name::ROOTS, Directory::Roots)?;
        let manifests = open_directory(&retention, pool_name::MANIFESTS, Directory::Manifests)?;
        Ok(Self {
            root,
            retention,
            roots,
            manifests,
            namespace: None,
            liveness_generation: None,
            retained_root: None,
            retained_manifest: None,
            root_stage: None,
            manifest_stage: None,
            head_stage: None,
            _lock: lock,
        })
    }

    /// Observes the published retention head and the manifest it selects.
    ///
    /// Returns `None` when no retention head has been published. This
    /// synchronous read performs no protocol mutation and does not consult
    /// retained stages; callers decode the returned bytes to plan the next
    /// transition, then let publication revalidate them under authority.
    ///
    /// # Errors
    ///
    /// Returns the exact open, kind, length, decode, or cross-check refusal.
    pub fn observe_current(&self) -> io::Result<Option<ObservedRetentionState>> {
        filesystem_retention_current::observe(&self.retention, &self.manifests)
    }

    pub(super) fn namespace(&self) -> io::Result<&Dir> {
        self.namespace
            .as_ref()
            .ok_or_else(|| invalid_data("retention root namespace was not admitted"))
    }

    pub(super) fn take_root_stage(&mut self) -> io::Result<FilesystemRetentionStage> {
        self.root_stage
            .take()
            .ok_or_else(|| invalid_data("retention root stage was not retained"))
    }

    pub(super) fn take_manifest_stage(&mut self) -> io::Result<FilesystemRetentionStage> {
        self.manifest_stage
            .take()
            .ok_or_else(|| invalid_data("retention manifest stage was not retained"))
    }

    pub(super) fn take_head_stage(&mut self) -> io::Result<FilesystemRetentionStage> {
        self.head_stage
            .take()
            .ok_or_else(|| invalid_data("retention head stage was not retained"))
    }

    pub(super) fn root_stage(&self) -> io::Result<&FilesystemRetentionStage> {
        self.root_stage
            .as_ref()
            .ok_or_else(|| invalid_data("retention root stage was not retained"))
    }

    pub(super) fn manifest_stage(&self) -> io::Result<&FilesystemRetentionStage> {
        self.manifest_stage
            .as_ref()
            .ok_or_else(|| invalid_data("retention manifest stage was not retained"))
    }

    pub(super) fn head_stage(&self) -> io::Result<&FilesystemRetentionStage> {
        self.head_stage
            .as_ref()
            .ok_or_else(|| invalid_data("retention head stage was not retained"))
    }
}

fn open_directory(parent: &Dir, name: &str, directory: Directory) -> Result<Dir, Error> {
    parent
        .open_dir_nofollow(name)
        .map_err(|source| Error::Directory { directory, source })
}
