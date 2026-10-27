//! This module owns the catalog stages reachable only after retained-closure admission.
//!
//! Version-two catalog publication executes the shared
//! [`FilesystemCatalogStages`] implementation through this type. It has no
//! public constructor and is reachable only through
//! [`RetainedCatalogPublicationStorage::catalog_stages`](crate::RetainedCatalogPublicationStorage::catalog_stages),
//! which requires a [`RetainedClosureAdmission`](crate::RetainedClosureAdmission)
//! that only the gate creates. The ungated
//! [`publish_catalog_generation`](crate::publish_catalog_generation) can
//! therefore never drive version-two stages.

use std::io;

use super::filesystem_catalog_stages::FilesystemCatalogStages;
use super::{
    AdmittedSegment, CanonicalCatalog, CanonicalPublicationHead, CatalogPublicationExpectation,
    CatalogPublicationPhase as Phase, CatalogPublicationReadiness, CatalogPublicationStorage,
    CatalogSnapshot, ChecksummedCatalog, SegmentPublication,
};

/// Where a repository-task observer runs relative to one catalog transition.
#[cfg(any(test, feature = "repository-tasks"))]
#[doc(hidden)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CatalogPublicationBoundary {
    /// Before the transition begins.
    Before,
    /// After the transition returned successfully.
    After,
}

/// Repository-task observer of version-two catalog transitions.
///
/// Returning an error fails the transition at that boundary exactly as a
/// storage failure would; the crash matrix blocks inside it for process death.
#[cfg(any(test, feature = "repository-tasks"))]
#[doc(hidden)]
pub type CatalogPublicationObserver =
    Box<dyn FnMut(Phase, CatalogPublicationBoundary) -> io::Result<()>>;

/// Version-two catalog protocol transitions guarded by retained-closure admission.
///
/// Every transition is the shared version-one stage implementation, executed
/// under the version-two publisher's writer authority. Fields are private and
/// no constructor is public.
#[must_use]
pub struct FilesystemVersionTwoCatalogStages {
    stages: FilesystemCatalogStages,
    #[cfg(any(test, feature = "repository-tasks"))]
    observer: Option<CatalogPublicationObserver>,
}

impl FilesystemVersionTwoCatalogStages {
    pub(super) const fn new(stages: FilesystemCatalogStages) -> Self {
        Self {
            stages,
            #[cfg(any(test, feature = "repository-tasks"))]
            observer: None,
        }
    }

    pub(super) const fn shared(&self) -> &FilesystemCatalogStages {
        &self.stages
    }

    #[cfg(any(test, feature = "repository-tasks"))]
    pub(super) fn observe(&mut self, observer: CatalogPublicationObserver) {
        self.observer = Some(observer);
    }

    /// Runs one shared transition, notifying a repository-task observer around it.
    #[cfg(any(test, feature = "repository-tasks"))]
    fn transition<T>(
        &mut self,
        phase: Phase,
        operation: impl FnOnce(&mut FilesystemCatalogStages) -> io::Result<T>,
    ) -> io::Result<T> {
        self.notify(phase, CatalogPublicationBoundary::Before)?;
        let value = operation(&mut self.stages)?;
        self.notify(phase, CatalogPublicationBoundary::After)?;
        Ok(value)
    }

    /// Runs one shared transition; production builds observe no boundary.
    #[cfg(not(any(test, feature = "repository-tasks")))]
    fn transition<T>(
        &mut self,
        _phase: Phase,
        operation: impl FnOnce(&mut FilesystemCatalogStages) -> io::Result<T>,
    ) -> io::Result<T> {
        operation(&mut self.stages)
    }

    #[cfg(any(test, feature = "repository-tasks"))]
    fn notify(&mut self, phase: Phase, boundary: CatalogPublicationBoundary) -> io::Result<()> {
        self.observer
            .as_mut()
            .map_or(Ok(()), |observer| observer(phase, boundary))
    }
}

impl CatalogPublicationStorage for FilesystemVersionTwoCatalogStages {
    fn verify_current(
        &mut self,
        expected: CatalogPublicationExpectation,
        candidate: &CatalogSnapshot<'_, '_, '_>,
        segment: &SegmentPublication<'_, '_>,
    ) -> io::Result<CatalogPublicationReadiness> {
        self.transition(Phase::VerifyCurrent, |stages| {
            stages.verify_current(expected, candidate, segment)
        })
    }

    fn link_segment(&mut self, segment: &AdmittedSegment<'_>) -> io::Result<()> {
        self.transition(Phase::LinkSegment, |stages| stages.link_segment(segment))
    }

    fn verify_segment_pool(&mut self, segment: &AdmittedSegment<'_>) -> io::Result<()> {
        self.transition(Phase::VerifySegmentPool, |stages| {
            stages.verify_segment_pool(segment)
        })
    }

    fn synchronize_segments(&mut self) -> io::Result<()> {
        self.transition(Phase::SynchronizeSegments, |stages| {
            stages.synchronize_segments()
        })
    }

    fn remove_segment_stage(&mut self) -> io::Result<()> {
        self.transition(Phase::RemoveSegmentStage, |stages| {
            stages.remove_segment_stage()
        })
    }

    fn synchronize_staging_after_segment(&mut self) -> io::Result<()> {
        self.transition(Phase::SynchronizeStagingAfterSegment, |stages| {
            stages.synchronize_staging_after_segment()
        })
    }

    fn create_catalog_stage(&mut self) -> io::Result<()> {
        self.transition(Phase::CreateCatalogStage, |stages| {
            stages.create_catalog_stage()
        })
    }

    fn write_catalog(&mut self, catalog: &CanonicalCatalog) -> io::Result<()> {
        self.transition(Phase::WriteCatalog, |stages| stages.write_catalog(catalog))
    }

    fn flush_catalog(&mut self) -> io::Result<()> {
        self.transition(
            Phase::FlushCatalog,
            CatalogPublicationStorage::flush_catalog,
        )
    }

    fn synchronize_catalog(&mut self) -> io::Result<()> {
        self.transition(Phase::SynchronizeCatalog, |stages| {
            stages.synchronize_catalog()
        })
    }

    fn link_catalog(&mut self, catalog: ChecksummedCatalog<'_>) -> io::Result<()> {
        self.transition(Phase::LinkCatalog, |stages| stages.link_catalog(catalog))
    }

    fn verify_catalog_pool(&mut self, catalog: ChecksummedCatalog<'_>) -> io::Result<()> {
        self.transition(Phase::VerifyCatalogPool, |stages| {
            stages.verify_catalog_pool(catalog)
        })
    }

    fn synchronize_catalogs(&mut self) -> io::Result<()> {
        self.transition(Phase::SynchronizeCatalogs, |stages| {
            stages.synchronize_catalogs()
        })
    }

    fn remove_catalog_stage(&mut self) -> io::Result<()> {
        self.transition(Phase::RemoveCatalogStage, |stages| {
            stages.remove_catalog_stage()
        })
    }

    fn synchronize_staging_after_catalog(&mut self) -> io::Result<()> {
        self.transition(Phase::SynchronizeStagingAfterCatalog, |stages| {
            stages.synchronize_staging_after_catalog()
        })
    }

    fn create_head_stage(&mut self) -> io::Result<()> {
        self.transition(
            Phase::CreateHeadStage,
            CatalogPublicationStorage::create_head_stage,
        )
    }

    fn write_head(&mut self, head: &CanonicalPublicationHead) -> io::Result<()> {
        self.transition(Phase::WriteHead, |stages| stages.write_head(head))
    }

    fn flush_head(&mut self) -> io::Result<()> {
        self.transition(Phase::FlushHead, CatalogPublicationStorage::flush_head)
    }

    fn synchronize_head(&mut self) -> io::Result<()> {
        self.transition(
            Phase::SynchronizeHead,
            CatalogPublicationStorage::synchronize_head,
        )
    }

    fn verify_head_view(
        &mut self,
        head: &CanonicalPublicationHead,
        snapshot: &CatalogSnapshot<'_, '_, '_>,
    ) -> io::Result<()> {
        self.transition(Phase::VerifyHeadView, |stages| {
            stages.verify_head_view(head, snapshot)
        })
    }

    fn replace_head(&mut self) -> io::Result<()> {
        self.transition(Phase::ReplaceHead, CatalogPublicationStorage::replace_head)
    }

    fn synchronize_root(&mut self) -> io::Result<()> {
        self.transition(
            Phase::SynchronizeRoot,
            CatalogPublicationStorage::synchronize_root,
        )
    }
}
