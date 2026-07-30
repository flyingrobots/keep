//! This boundary module owns blocking retention publication durability capabilities.

use std::io;

use super::{
    AdmittedRetentionRoot, CanonicalRetentionHead, CanonicalRetentionManifest,
    RetentionNamespaceAdmission,
};

/// Blocking storage capabilities for one writer-locked retention publication.
///
/// An implementation must retain exclusive writer authority and one pinned
/// store root for the complete operation. Each method corresponds to one
/// [`RetentionPublicationPhase`](super::RetentionPublicationPhase) and must not
/// report success before the named durability and verification obligations are
/// complete.
pub trait RetentionPublicationStorage {
    /// Exclusively creates and completely writes the canonical root stage.
    ///
    /// # Errors
    ///
    /// Returns the exact creation, write, or flush failure.
    fn write_root_stage(&mut self, root: &AdmittedRetentionRoot<'_>) -> io::Result<()>;

    /// Synchronizes the complete root stage.
    ///
    /// # Errors
    ///
    /// Returns the exact file-synchronization failure.
    fn synchronize_root_stage(&mut self) -> io::Result<()>;

    /// Creates or exactly admits the candidate's digest-named root namespace.
    ///
    /// # Errors
    ///
    /// Returns the exact namespace creation or admission failure.
    fn admit_root_namespace(
        &mut self,
        root: &AdmittedRetentionRoot<'_>,
    ) -> io::Result<RetentionNamespaceAdmission>;

    /// Synchronizes `retention/roots` after namespace creation.
    ///
    /// # Errors
    ///
    /// Returns the exact directory-synchronization failure.
    fn synchronize_roots_after_namespace(&mut self) -> io::Result<()>;

    /// Links and completely verifies the immutable root-pool entry.
    ///
    /// # Errors
    ///
    /// Returns the exact link, reopen, or verification failure.
    fn link_root(&mut self, root: &AdmittedRetentionRoot<'_>) -> io::Result<()>;

    /// Synchronizes the candidate's digest-named root namespace.
    ///
    /// # Errors
    ///
    /// Returns the exact directory-synchronization failure.
    fn synchronize_root_namespace(&mut self, root: &AdmittedRetentionRoot<'_>) -> io::Result<()>;

    /// Exclusively creates and completely writes the canonical manifest stage.
    ///
    /// # Errors
    ///
    /// Returns the exact creation, write, or flush failure.
    fn write_manifest_stage(&mut self, manifest: &CanonicalRetentionManifest) -> io::Result<()>;

    /// Synchronizes the complete manifest stage.
    ///
    /// # Errors
    ///
    /// Returns the exact file-synchronization failure.
    fn synchronize_manifest_stage(&mut self) -> io::Result<()>;

    /// Links and completely verifies the immutable manifest-pool entry.
    ///
    /// # Errors
    ///
    /// Returns the exact link, reopen, or verification failure.
    fn link_manifest(&mut self, manifest: &CanonicalRetentionManifest) -> io::Result<()>;

    /// Synchronizes the immutable manifest pool.
    ///
    /// # Errors
    ///
    /// Returns the exact directory-synchronization failure.
    fn synchronize_manifest_pool(&mut self) -> io::Result<()>;

    /// Exclusively creates and completely writes the canonical head stage.
    ///
    /// # Errors
    ///
    /// Returns the exact creation, write, or flush failure.
    fn write_head_stage(&mut self, head: &CanonicalRetentionHead) -> io::Result<()>;

    /// Synchronizes the complete head stage.
    ///
    /// # Errors
    ///
    /// Returns the exact file-synchronization failure.
    fn synchronize_head_stage(&mut self) -> io::Result<()>;

    /// Atomically replaces `retention/HEAD` with the synchronized head stage.
    ///
    /// # Errors
    ///
    /// Returns the exact replacement failure.
    fn replace_head(&mut self) -> io::Result<()>;

    /// Synchronizes `retention` after head replacement.
    ///
    /// # Errors
    ///
    /// Returns the exact directory-synchronization failure.
    fn synchronize_retention_namespace(&mut self) -> io::Result<()>;

    /// Removes only the retained root stage.
    ///
    /// # Errors
    ///
    /// Returns the exact removal failure.
    fn remove_root_stage(&mut self) -> io::Result<()>;

    /// Removes only the retained manifest stage.
    ///
    /// # Errors
    ///
    /// Returns the exact removal failure.
    fn remove_manifest_stage(&mut self) -> io::Result<()>;

    /// Synchronizes `retention` after both stage removals.
    ///
    /// # Errors
    ///
    /// Returns the exact directory-synchronization failure.
    fn synchronize_cleanup(&mut self) -> io::Result<()>;
}
