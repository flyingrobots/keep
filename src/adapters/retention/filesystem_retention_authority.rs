//! This module owns exact writer-locked filesystem retention authority.

use std::io;

use cap_std::fs::Dir;

use super::filesystem_retention_attempt::PublicationAttempt;
use super::filesystem_retention_authority_error::{
    FilesystemRetentionAuthorityError as Error, RetentionAuthorityDirectory as Directory,
};
use super::filesystem_retention_current::{self, ObservedRetentionState};
use crate::adapters::{FilesystemVersionTwoAdmission, FilesystemWriterLock};

/// Exclusive authority to publish retention transitions on one pinned root.
///
/// The authority retains the admitted writer lock and pinned root,
/// `retention`, `retention/roots`, and `retention/manifests` capabilities for
/// its entire lifetime. When passed to
/// [`execute_retention_publication`](crate::execute_retention_publication) its
/// [`RetentionPublicationStorage`](super::RetentionPublicationStorage)
/// implementation executes only the forward publication protocol from an
/// exactly admitted current state. Every coordinate one run retains between
/// phases, including opened stage handles, lives on one publication attempt
/// that current-state verification creates and the next verification or
/// cleanup discards. It performs synchronous capability-relative I/O and uses
/// neither a network nor an asynchronous runtime. Reopening a retained stage
/// prefix remains a separate recovery boundary.
#[must_use]
pub struct FilesystemRetentionPublicationAuthority {
    pub(super) root: Dir,
    pub(super) retention: Dir,
    pub(super) roots: Dir,
    pub(super) manifests: Dir,
    pub(super) attempt: Option<PublicationAttempt>,
    _lock: FilesystemWriterLock,
}

impl FilesystemRetentionPublicationAuthority {
    /// Pins one admitted version-two root for retention publication.
    ///
    /// Only [`FilesystemVersionTwoAdmission`] is accepted, so version-one
    /// writer authority can never reach retention publication; the type
    /// system refuses it:
    ///
    /// ```compile_fail
    /// fn publish(admission: keep::FilesystemPlatformAdmission) {
    ///     let _ = keep::FilesystemRetentionPublicationAuthority::open(admission);
    /// }
    /// ```
    ///
    /// This synchronous constructor opens pinned directory capabilities but
    /// materializes no record bodies and performs no protocol mutation.
    ///
    /// # Errors
    ///
    /// Returns [`FilesystemRetentionAuthorityError`](super::FilesystemRetentionAuthorityError)
    /// when the root capability cannot be cloned. The retention namespace and
    /// both immutable pools arrive already pinned by admission.
    pub fn open(admission: FilesystemVersionTwoAdmission) -> Result<Self, Error> {
        let (lock, retention, roots, manifests) = admission.into_parts();
        let root = lock.clone_directory().map_err(|source| Error::Directory {
            directory: Directory::Root,
            source,
        })?;
        Ok(Self {
            root,
            retention,
            roots,
            manifests,
            attempt: None,
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
}
