//! This module owns writer-locked filesystem catalog publication state.

use std::io;

use cap_std::fs::{Dir, File};

use super::filesystem_publisher_authority::FilesystemPublisherAuthority;
use super::{
    AdmittedSegment, CatalogRestartPolicy, ClosedSegment, FilesystemPlatformAdmission,
    FilesystemSegmentStage, FilesystemWriterLock, SealedSegment, SegmentPublication,
    SegmentPublicationError, SegmentStageCreateError, sync_capable_directory,
};
#[cfg(feature = "repository-tasks")]
use super::{CanonicalCatalog, CanonicalPublicationHead, filesystem_catalog_catalog};

pub(super) const CURRENT_SEGMENT: &str = "current.seg";
pub(super) const CURRENT_CATALOG: &str = "current.cat";
pub(super) const HEAD: &str = "HEAD";
pub(super) const NEXT_HEAD: &str = "head.next";

/// Exclusive filesystem authority for one catalog publication at a time.
///
/// The publisher owns the writer lock and pinned root, staging, segment-pool,
/// and catalog-pool directory capabilities until it is dropped. Dropping it
/// closes open stages and directory capabilities before releasing the writer
/// lock, but never publishes, removes, truncates, or repairs protocol state.
///
/// Construction consumes a [`FilesystemPlatformAdmission`] proof created by
/// initializing a new store or reopening a completely published store.
#[must_use]
pub struct FilesystemCatalogPublisher {
    pub(super) root: Dir,
    pub(super) staging: Dir,
    pub(super) segments: Dir,
    pub(super) catalogs: Dir,
    pub(super) policy: CatalogRestartPolicy,
    pub(super) authority: FilesystemPublisherAuthority,
    pub(super) catalog_stage: Option<File>,
    pub(super) head_stage: Option<File>,
    // Fields drop in declaration order. Writer authority must outlive every
    // directory capability and retained writable stage.
    pub(super) _lock: FilesystemWriterLock,
}

impl FilesystemCatalogPublisher {
    /// Pins canonical publication directories under admitted writer authority.
    ///
    /// # Errors
    ///
    /// Returns the exact root-clone, no-follow directory-open, or
    /// directory-inspection failure. A namespace entry that is not a directory
    /// returns [`io::ErrorKind::NotADirectory`]. A failure drops `lock` and
    /// therefore releases writer authority. Success allocates one ephemeral
    /// authority token that binds later stage selection to this publisher.
    pub fn open(
        admission: FilesystemPlatformAdmission,
        policy: CatalogRestartPolicy,
    ) -> io::Result<Self> {
        let lock = admission.into_lock();
        let pinned_root = lock.clone_directory()?;
        let root = sync_capable_directory::open(&pinned_root, ".")?;
        let staging = sync_capable_directory::open(&root, "staging")?;
        let segments = sync_capable_directory::open(&root, "segments")?;
        let catalogs = sync_capable_directory::open(&root, "catalogs")?;
        Ok(Self {
            root,
            staging,
            segments,
            catalogs,
            policy,
            authority: FilesystemPublisherAuthority::new(),
            catalog_stage: None,
            head_stage: None,
            _lock: lock,
        })
    }

    /// Opens a publisher without the production platform-profile proof.
    ///
    /// Repository process-death tests use this after executing the production
    /// initialization protocol through [`crate::RepositoryInitializationStorage`].
    ///
    /// # Errors
    ///
    /// Returns the same pinned-directory admission failures as [`Self::open`].
    #[cfg(feature = "repository-tasks")]
    #[doc(hidden)]
    pub fn open_unchecked_for_repository_tasks(
        lock: FilesystemWriterLock,
        policy: CatalogRestartPolicy,
    ) -> io::Result<Self> {
        Self::open(
            FilesystemPlatformAdmission::unchecked_for_repository_tasks(lock),
            policy,
        )
    }

    #[cfg(test)]
    pub(super) fn open_unchecked_for_tests(
        lock: FilesystemWriterLock,
        policy: CatalogRestartPolicy,
    ) -> io::Result<Self> {
        Self::open(
            FilesystemPlatformAdmission::unchecked_for_tests(lock),
            policy,
        )
    }

    /// Exclusively creates `staging/current.seg` under this writer authority.
    ///
    /// The returned stage borrows this publisher until the stage is dropped or
    /// consumed by [`crate::StagedSegment`] and explicitly closed after
    /// sealing. Creation performs blocking, capability-relative filesystem I/O.
    ///
    /// # Errors
    ///
    /// Returns [`SegmentStageCreateError`] without opening or truncating an
    /// existing filesystem entry.
    pub fn create_segment_stage(
        &self,
    ) -> Result<FilesystemSegmentStage<'_>, SegmentStageCreateError> {
        FilesystemSegmentStage::create(self)
    }

    /// Closes and selects one synchronized stage created by this publisher.
    ///
    /// The returned selection remains bound to this exact publisher instance
    /// and cannot authorize a metadata-equivalent retained stage owned by
    /// another publisher or storage implementation.
    ///
    /// # Errors
    ///
    /// Returns [`SegmentPublicationError::PublisherAuthority`] when `sealed`
    /// was created by another publisher. Other variants preserve exact
    /// closed-stage to admitted-segment disagreements.
    pub fn select_segment<'selection, 'records>(
        &self,
        sealed: SealedSegment<FilesystemSegmentStage<'_>>,
        admitted: &'selection AdmittedSegment<'records>,
    ) -> Result<SegmentPublication<'selection, 'records>, SegmentPublicationError> {
        let (stage, record_count, segment_length, digest) = sealed.into_parts();
        let authority = stage.close();
        if !self.authority.matches(&authority) {
            return Err(SegmentPublicationError::PublisherAuthority);
        }
        SegmentPublication::one_bound(
            ClosedSegment::admitted(record_count, segment_length, digest),
            admitted,
            authority,
        )
    }

    /// Writes a strict prefix through the production catalog-stage adapter.
    ///
    /// # Errors
    ///
    /// Returns the exact missing-stage, prefix-bound, or write failure.
    #[cfg(feature = "repository-tasks")]
    #[doc(hidden)]
    pub fn write_catalog_prefix_for_repository_tasks(
        &mut self,
        catalog: &CanonicalCatalog,
        prefix: usize,
    ) -> io::Result<()> {
        filesystem_catalog_catalog::write_prefix(self, catalog, prefix)
    }

    /// Writes a strict prefix through the production head-stage adapter.
    ///
    /// # Errors
    ///
    /// Returns the exact missing-stage, prefix-bound, or write failure.
    #[cfg(feature = "repository-tasks")]
    #[doc(hidden)]
    pub fn write_head_prefix_for_repository_tasks(
        &mut self,
        head: &CanonicalPublicationHead,
        prefix: usize,
    ) -> io::Result<()> {
        super::filesystem_catalog_head::write_prefix(self, head, prefix)
    }
}
