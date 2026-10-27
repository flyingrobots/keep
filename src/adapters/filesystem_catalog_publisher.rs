//! This module owns writer-locked catalog publication state.

use std::io;

use super::filesystem_catalog_stages::FilesystemCatalogStages;
#[cfg(feature = "repository-tasks")]
use super::filesystem_retained_catalog_stages::CatalogPublicationObserver;
use super::retention::FilesystemRetainedEvidence;
use super::{
    AdmittedSegment, CatalogRestartPolicy, FilesystemRetainedCatalogStages, FilesystemSegmentStage,
    FilesystemStoreAdmission, FilesystemWriterLock, SealedSegment, SegmentPublication,
    SegmentPublicationError, SegmentStageCreateError,
};

/// Exclusive store authority to ingest one segment and publish a catalog.
///
/// The publisher owns the writer lock, the pinned store root, the catalog
/// protocol directories, and the `retention`, `retention/roots`, and
/// `retention/manifests` capabilities that store admission pinned. It
/// publishes only through
/// [`publish_retained_catalog_generation`](crate::publish_retained_catalog_generation):
/// it implements [`RetainedCatalogPublicationStorage`](crate::RetainedCatalogPublicationStorage)
/// and not the ungated port, so the type system refuses
/// [`publish_catalog_generation`](crate::publish_catalog_generation):
///
/// ```compile_fail
/// fn publish(
///     mut publisher: keep::FilesystemCatalogPublisher,
///     catalog: &keep::CanonicalCatalog,
/// ) {
///     let _ = keep::publish_catalog_generation(
///         &mut publisher,
///         keep::CatalogPublicationExpectation::uninitialized(),
///         keep::SegmentPublication::none(),
///         catalog,
///         &[],
///     );
/// }
/// ```
///
/// Publication takes no reader fence; readers keep their views beside it.
/// Dropping the publisher closes open stages and every directory capability
/// before releasing the writer lock, but never publishes, removes, truncates,
/// or repairs protocol state.
#[must_use]
pub struct FilesystemCatalogPublisher {
    pub(super) evidence: FilesystemRetainedEvidence,
    pub(super) stages: FilesystemRetainedCatalogStages,
    // Fields drop in declaration order. Writer authority must outlive every
    // directory capability and retained writable stage.
    _lock: FilesystemWriterLock,
}

impl FilesystemCatalogPublisher {
    /// Pins the catalog and retention directories under admitted writer authority.
    ///
    /// Only [`FilesystemStoreAdmission`] is accepted, so a bare writer
    /// lock, which proves neither the namespace nor the `FORMAT` marker, cannot
    /// reach this publisher:
    ///
    /// ```compile_fail
    /// fn open(lock: keep::FilesystemWriterLock, policy: keep::CatalogRestartPolicy) {
    ///     let _ = keep::FilesystemCatalogPublisher::open(lock, policy);
    /// }
    /// ```
    ///
    /// The synchronous call performs no protocol mutation and reads no record.
    ///
    /// # Errors
    ///
    /// Returns the exact root-clone, no-follow directory-open, or
    /// directory-inspection failure. A protocol entry that is not a directory
    /// returns [`io::ErrorKind::NotADirectory`]. A failure releases the writer
    /// lock.
    pub fn open(
        admission: FilesystemStoreAdmission,
        policy: CatalogRestartPolicy,
    ) -> io::Result<Self> {
        let (lock, retention, roots, manifests) = admission.into_parts();
        let pinned_root = lock.clone_directory()?;
        let stages = FilesystemCatalogStages::open(&pinned_root, policy)?;
        Ok(Self {
            evidence: FilesystemRetainedEvidence::new(pinned_root, retention, roots, manifests),
            stages: FilesystemRetainedCatalogStages::new(stages),
            _lock: lock,
        })
    }

    /// Exclusively creates `staging/current.seg` under this writer authority.
    ///
    /// The stage borrows this publisher until it is dropped or sealed and
    /// selected. Creation performs blocking, capability-relative I/O.
    ///
    /// # Errors
    ///
    /// Returns [`SegmentStageCreateError`] without opening or truncating an
    /// existing filesystem entry.
    pub fn create_segment_stage(
        &self,
    ) -> Result<FilesystemSegmentStage<'_>, SegmentStageCreateError> {
        self.stages.shared().create_segment_stage()
    }

    /// Closes and selects one synchronized stage created by this publisher.
    ///
    /// # Errors
    ///
    /// Returns [`SegmentPublicationError::PublisherAuthority`] when `sealed`
    /// was created by another publisher.
    /// Other variants preserve exact closed-stage to admitted-segment
    /// disagreements.
    pub fn select_segment<'selection, 'records>(
        &self,
        sealed: SealedSegment<FilesystemSegmentStage<'_>>,
        admitted: &'selection AdmittedSegment<'records>,
    ) -> Result<SegmentPublication<'selection, 'records>, SegmentPublicationError> {
        self.stages.shared().select_segment(sealed, admitted)
    }

    /// Installs an observer around every catalog transition for repository tasks.
    ///
    /// The crash matrix blocks inside the observer for process death; an
    /// observer error fails the transition at that boundary.
    #[cfg(feature = "repository-tasks")]
    #[doc(hidden)]
    pub fn observe_catalog_phases_for_repository_tasks(
        &mut self,
        observer: CatalogPublicationObserver,
    ) {
        self.stages.observe(observer);
    }
}
