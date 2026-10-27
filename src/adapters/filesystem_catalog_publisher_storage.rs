//! This module binds version-one publication authority to the shared stages.
//!
//! Every transition forwards unchanged to
//! [`FilesystemCatalogStages`](super::filesystem_catalog_stages::FilesystemCatalogStages),
//! so the version-one port and the version-two retained-closure port execute
//! one implementation.

use std::io;

use super::{
    AdmittedSegment, CanonicalCatalog, CanonicalPublicationHead, CatalogPublicationExpectation,
    CatalogPublicationReadiness, CatalogPublicationStorage, CatalogSnapshot, ChecksummedCatalog,
    FilesystemCatalogPublisher, SegmentPublication,
};

impl CatalogPublicationStorage for FilesystemCatalogPublisher {
    fn verify_current(
        &mut self,
        expected: CatalogPublicationExpectation,
        candidate: &CatalogSnapshot<'_, '_, '_>,
        segment: &SegmentPublication<'_, '_>,
    ) -> io::Result<CatalogPublicationReadiness> {
        self.stages.verify_current(expected, candidate, segment)
    }

    fn link_segment(&mut self, segment: &AdmittedSegment<'_>) -> io::Result<()> {
        self.stages.link_segment(segment)
    }

    fn verify_segment_pool(&mut self, segment: &AdmittedSegment<'_>) -> io::Result<()> {
        self.stages.verify_segment_pool(segment)
    }

    fn synchronize_segments(&mut self) -> io::Result<()> {
        self.stages.synchronize_segments()
    }

    fn remove_segment_stage(&mut self) -> io::Result<()> {
        self.stages.remove_segment_stage()
    }

    fn synchronize_staging_after_segment(&mut self) -> io::Result<()> {
        self.stages.synchronize_staging_after_segment()
    }

    fn create_catalog_stage(&mut self) -> io::Result<()> {
        self.stages.create_catalog_stage()
    }

    fn write_catalog(&mut self, catalog: &CanonicalCatalog) -> io::Result<()> {
        self.stages.write_catalog(catalog)
    }

    fn flush_catalog(&mut self) -> io::Result<()> {
        self.stages.flush_catalog()
    }

    fn synchronize_catalog(&mut self) -> io::Result<()> {
        self.stages.synchronize_catalog()
    }

    fn link_catalog(&mut self, catalog: ChecksummedCatalog<'_>) -> io::Result<()> {
        self.stages.link_catalog(catalog)
    }

    fn verify_catalog_pool(&mut self, catalog: ChecksummedCatalog<'_>) -> io::Result<()> {
        self.stages.verify_catalog_pool(catalog)
    }

    fn synchronize_catalogs(&mut self) -> io::Result<()> {
        self.stages.synchronize_catalogs()
    }

    fn remove_catalog_stage(&mut self) -> io::Result<()> {
        self.stages.remove_catalog_stage()
    }

    fn synchronize_staging_after_catalog(&mut self) -> io::Result<()> {
        self.stages.synchronize_staging_after_catalog()
    }

    fn create_head_stage(&mut self) -> io::Result<()> {
        self.stages.create_head_stage()
    }

    fn write_head(&mut self, head: &CanonicalPublicationHead) -> io::Result<()> {
        self.stages.write_head(head)
    }

    fn flush_head(&mut self) -> io::Result<()> {
        self.stages.flush_head()
    }

    fn synchronize_head(&mut self) -> io::Result<()> {
        self.stages.synchronize_head()
    }

    fn verify_head_view(
        &mut self,
        head: &CanonicalPublicationHead,
        snapshot: &CatalogSnapshot<'_, '_, '_>,
    ) -> io::Result<()> {
        self.stages.verify_head_view(head, snapshot)
    }

    fn replace_head(&mut self) -> io::Result<()> {
        self.stages.replace_head()
    }

    fn synchronize_root(&mut self) -> io::Result<()> {
        self.stages.synchronize_root()
    }
}
