//! This module owns crash injection around production catalog publication.

use std::io;

use keep::{
    AdmittedSegment, CanonicalCatalog, CanonicalPublicationHead, CatalogPublicationExpectation,
    CatalogPublicationReadiness, CatalogPublicationStorage, CatalogSnapshot, ChecksummedCatalog,
    FilesystemCatalogPublisher, SegmentPublication,
};
use xtask::{DurabilityCrashPoint, DurabilityCrashPosition};

use super::control::{CrashControl, DuringTiming};

const CATALOG_INTERRUPTION: usize = 176;
const HEAD_INTERRUPTION: usize = 64;

pub(super) struct CrashPublicationStorage<'control> {
    inner: FilesystemCatalogPublisher,
    control: &'control mut CrashControl,
}

impl<'control> CrashPublicationStorage<'control> {
    pub(super) const fn new(
        inner: FilesystemCatalogPublisher,
        control: &'control mut CrashControl,
    ) -> Self {
        Self { inner, control }
    }
}

impl CatalogPublicationStorage for CrashPublicationStorage<'_> {
    fn verify_current(
        &mut self,
        expected: CatalogPublicationExpectation,
        candidate: &CatalogSnapshot<'_, '_, '_>,
        segment: &SegmentPublication<'_, '_>,
    ) -> io::Result<CatalogPublicationReadiness> {
        self.inner.verify_current(expected, candidate, segment)
    }

    fn link_segment(&mut self, segment: &AdmittedSegment<'_>) -> io::Result<()> {
        execute(
            &mut self.inner,
            self.control,
            DurabilityCrashPoint::LinkSegment,
            DuringTiming::After,
            |inner| inner.link_segment(segment),
        )
    }

    fn verify_segment_pool(&mut self, segment: &AdmittedSegment<'_>) -> io::Result<()> {
        self.inner.verify_segment_pool(segment)
    }

    fn synchronize_segments(&mut self) -> io::Result<()> {
        execute(
            &mut self.inner,
            self.control,
            DurabilityCrashPoint::SynchronizeSegmentPool,
            DuringTiming::Before,
            FilesystemCatalogPublisher::synchronize_segments,
        )
    }

    fn remove_segment_stage(&mut self) -> io::Result<()> {
        execute(
            &mut self.inner,
            self.control,
            DurabilityCrashPoint::RemoveSegmentStage,
            DuringTiming::After,
            FilesystemCatalogPublisher::remove_segment_stage,
        )
    }

    fn synchronize_staging_after_segment(&mut self) -> io::Result<()> {
        execute(
            &mut self.inner,
            self.control,
            DurabilityCrashPoint::SynchronizeStagingAfterSegment,
            DuringTiming::Before,
            FilesystemCatalogPublisher::synchronize_staging_after_segment,
        )
    }

    fn create_catalog_stage(&mut self) -> io::Result<()> {
        execute(
            &mut self.inner,
            self.control,
            DurabilityCrashPoint::CreateCatalogStage,
            DuringTiming::After,
            FilesystemCatalogPublisher::create_catalog_stage,
        )
    }

    fn write_catalog(&mut self, catalog: &CanonicalCatalog) -> io::Result<()> {
        execute_write(
            &mut self.inner,
            self.control,
            DurabilityCrashPoint::WriteCatalog,
            |inner| inner.write_catalog(catalog),
            |inner| inner.write_catalog_prefix_for_repository_tasks(catalog, CATALOG_INTERRUPTION),
        )
    }

    fn flush_catalog(&mut self) -> io::Result<()> {
        execute(
            &mut self.inner,
            self.control,
            DurabilityCrashPoint::FlushCatalog,
            DuringTiming::Before,
            FilesystemCatalogPublisher::flush_catalog,
        )
    }

    fn synchronize_catalog(&mut self) -> io::Result<()> {
        execute(
            &mut self.inner,
            self.control,
            DurabilityCrashPoint::SynchronizeCatalog,
            DuringTiming::Before,
            FilesystemCatalogPublisher::synchronize_catalog,
        )
    }

    fn link_catalog(&mut self, catalog: ChecksummedCatalog<'_>) -> io::Result<()> {
        execute(
            &mut self.inner,
            self.control,
            DurabilityCrashPoint::LinkCatalog,
            DuringTiming::After,
            |inner| inner.link_catalog(catalog),
        )
    }

    fn verify_catalog_pool(&mut self, catalog: ChecksummedCatalog<'_>) -> io::Result<()> {
        self.inner.verify_catalog_pool(catalog)
    }

    fn synchronize_catalogs(&mut self) -> io::Result<()> {
        execute(
            &mut self.inner,
            self.control,
            DurabilityCrashPoint::SynchronizeCatalogPool,
            DuringTiming::Before,
            FilesystemCatalogPublisher::synchronize_catalogs,
        )
    }

    fn remove_catalog_stage(&mut self) -> io::Result<()> {
        execute(
            &mut self.inner,
            self.control,
            DurabilityCrashPoint::RemoveCatalogStage,
            DuringTiming::After,
            FilesystemCatalogPublisher::remove_catalog_stage,
        )
    }

    fn synchronize_staging_after_catalog(&mut self) -> io::Result<()> {
        execute(
            &mut self.inner,
            self.control,
            DurabilityCrashPoint::SynchronizeStagingAfterCatalog,
            DuringTiming::Before,
            FilesystemCatalogPublisher::synchronize_staging_after_catalog,
        )
    }

    fn create_head_stage(&mut self) -> io::Result<()> {
        execute(
            &mut self.inner,
            self.control,
            DurabilityCrashPoint::CreateHeadStage,
            DuringTiming::After,
            FilesystemCatalogPublisher::create_head_stage,
        )
    }

    fn write_head(&mut self, head: &CanonicalPublicationHead) -> io::Result<()> {
        execute_write(
            &mut self.inner,
            self.control,
            DurabilityCrashPoint::WriteHead,
            |inner| inner.write_head(head),
            |inner| inner.write_head_prefix_for_repository_tasks(head, HEAD_INTERRUPTION),
        )
    }

    fn flush_head(&mut self) -> io::Result<()> {
        execute(
            &mut self.inner,
            self.control,
            DurabilityCrashPoint::FlushHead,
            DuringTiming::Before,
            FilesystemCatalogPublisher::flush_head,
        )
    }

    fn synchronize_head(&mut self) -> io::Result<()> {
        execute(
            &mut self.inner,
            self.control,
            DurabilityCrashPoint::SynchronizeHead,
            DuringTiming::Before,
            FilesystemCatalogPublisher::synchronize_head,
        )
    }

    fn verify_head_view(
        &mut self,
        head: &CanonicalPublicationHead,
        snapshot: &CatalogSnapshot<'_, '_, '_>,
    ) -> io::Result<()> {
        self.inner.verify_head_view(head, snapshot)
    }

    fn replace_head(&mut self) -> io::Result<()> {
        execute(
            &mut self.inner,
            self.control,
            DurabilityCrashPoint::ReplaceHead,
            DuringTiming::After,
            FilesystemCatalogPublisher::replace_head,
        )
    }

    fn synchronize_root(&mut self) -> io::Result<()> {
        execute(
            &mut self.inner,
            self.control,
            DurabilityCrashPoint::SynchronizeRootAfterHead,
            DuringTiming::Before,
            FilesystemCatalogPublisher::synchronize_root,
        )
    }
}

fn execute<T>(
    inner: &mut FilesystemCatalogPublisher,
    control: &mut CrashControl,
    point: DurabilityCrashPoint,
    during: DuringTiming,
    operation: impl FnOnce(&mut FilesystemCatalogPublisher) -> io::Result<T>,
) -> io::Result<T> {
    control.before(point, during)?;
    let result = operation(inner)?;
    control.after(point, during)?;
    Ok(result)
}

fn execute_write(
    inner: &mut FilesystemCatalogPublisher,
    control: &mut CrashControl,
    point: DurabilityCrashPoint,
    complete: impl FnOnce(&mut FilesystemCatalogPublisher) -> io::Result<()>,
    interrupted: impl FnOnce(&mut FilesystemCatalogPublisher) -> io::Result<()>,
) -> io::Result<()> {
    match control.position(point) {
        None => complete(inner),
        Some(DurabilityCrashPosition::Before) => control.await_process_death(),
        Some(DurabilityCrashPosition::During) => {
            interrupted(inner)?;
            control.await_process_death()
        }
        Some(DurabilityCrashPosition::After) => {
            complete(inner)?;
            control.await_process_death()
        }
    }
}
