//! This module binds filesystem catalog transitions to the publication port.

use std::io;

use super::{
    AdmittedSegment, CanonicalCatalog, CanonicalPublicationHead, CatalogPublicationExpectation,
    CatalogPublicationReadiness, CatalogPublicationStorage, CatalogSnapshot, ChecksummedCatalog,
    FilesystemCatalogPublisher, SegmentPublication, filesystem_catalog_catalog,
    filesystem_catalog_current, filesystem_catalog_head, filesystem_catalog_segment,
};

impl CatalogPublicationStorage for FilesystemCatalogPublisher {
    fn verify_current(
        &mut self,
        expected: CatalogPublicationExpectation,
        candidate: &CatalogSnapshot<'_, '_, '_>,
        segment: &SegmentPublication<'_, '_>,
    ) -> io::Result<CatalogPublicationReadiness> {
        filesystem_catalog_current::verify(self, expected, candidate, segment)
    }

    fn link_segment(&mut self, segment: &AdmittedSegment<'_>) -> io::Result<()> {
        filesystem_catalog_segment::link(self, segment)
    }

    fn verify_segment_pool(&mut self, segment: &AdmittedSegment<'_>) -> io::Result<()> {
        filesystem_catalog_segment::verify_pool(self, segment)
    }

    fn synchronize_segments(&mut self) -> io::Result<()> {
        filesystem_catalog_segment::synchronize_pool(self)
    }

    fn remove_segment_stage(&mut self) -> io::Result<()> {
        filesystem_catalog_segment::remove_stage(self)
    }

    fn synchronize_staging_after_segment(&mut self) -> io::Result<()> {
        filesystem_catalog_segment::synchronize_staging(self)
    }

    fn create_catalog_stage(&mut self) -> io::Result<()> {
        filesystem_catalog_catalog::create_stage(self)
    }

    fn write_catalog(&mut self, catalog: &CanonicalCatalog) -> io::Result<()> {
        filesystem_catalog_catalog::write(self, catalog)
    }

    fn flush_catalog(&mut self) -> io::Result<()> {
        filesystem_catalog_catalog::flush(self)
    }

    fn synchronize_catalog(&mut self) -> io::Result<()> {
        filesystem_catalog_catalog::synchronize(self)
    }

    fn link_catalog(&mut self, catalog: ChecksummedCatalog<'_>) -> io::Result<()> {
        filesystem_catalog_catalog::link(self, catalog)
    }

    fn verify_catalog_pool(&mut self, catalog: ChecksummedCatalog<'_>) -> io::Result<()> {
        filesystem_catalog_catalog::verify_pool(self, catalog)
    }

    fn synchronize_catalogs(&mut self) -> io::Result<()> {
        filesystem_catalog_catalog::synchronize_pool(self)
    }

    fn remove_catalog_stage(&mut self) -> io::Result<()> {
        filesystem_catalog_catalog::remove_stage(self)
    }

    fn synchronize_staging_after_catalog(&mut self) -> io::Result<()> {
        filesystem_catalog_catalog::synchronize_staging(self)
    }

    fn create_head_stage(&mut self) -> io::Result<()> {
        filesystem_catalog_head::create_stage(self)
    }

    fn write_head(&mut self, head: &CanonicalPublicationHead) -> io::Result<()> {
        filesystem_catalog_head::write(self, head)
    }

    fn flush_head(&mut self) -> io::Result<()> {
        filesystem_catalog_head::flush(self)
    }

    fn synchronize_head(&mut self) -> io::Result<()> {
        filesystem_catalog_head::synchronize(self)
    }

    fn verify_head_view(
        &mut self,
        head: &CanonicalPublicationHead,
        snapshot: &CatalogSnapshot<'_, '_, '_>,
    ) -> io::Result<()> {
        filesystem_catalog_head::verify_view(self, head, snapshot)
    }

    fn replace_head(&mut self) -> io::Result<()> {
        filesystem_catalog_head::replace(self)
    }

    fn synchronize_root(&mut self) -> io::Result<()> {
        filesystem_catalog_head::synchronize_root(self)
    }
}
