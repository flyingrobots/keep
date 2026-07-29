//! Deterministic fault-recording catalog publication storage.

use std::io;

use keep::{
    AdmittedSegment, CanonicalCatalog, CanonicalPublicationHead, CatalogPublicationExpectation,
    CatalogPublicationPhase, CatalogPublicationReadiness, CatalogPublicationStorage,
    CatalogSnapshot, ChecksummedCatalog, SegmentPublication,
};

/// Exact complete publication order when one segment stage is present.
pub const EXPECTED_WITH_SEGMENT: &[CatalogPublicationPhase] = &[
    CatalogPublicationPhase::VerifyCurrent,
    CatalogPublicationPhase::LinkSegment,
    CatalogPublicationPhase::VerifySegmentPool,
    CatalogPublicationPhase::SynchronizeSegments,
    CatalogPublicationPhase::RemoveSegmentStage,
    CatalogPublicationPhase::SynchronizeStagingAfterSegment,
    CatalogPublicationPhase::CreateCatalogStage,
    CatalogPublicationPhase::WriteCatalog,
    CatalogPublicationPhase::FlushCatalog,
    CatalogPublicationPhase::SynchronizeCatalog,
    CatalogPublicationPhase::LinkCatalog,
    CatalogPublicationPhase::VerifyCatalogPool,
    CatalogPublicationPhase::SynchronizeCatalogs,
    CatalogPublicationPhase::RemoveCatalogStage,
    CatalogPublicationPhase::SynchronizeStagingAfterCatalog,
    CatalogPublicationPhase::CreateHeadStage,
    CatalogPublicationPhase::WriteHead,
    CatalogPublicationPhase::FlushHead,
    CatalogPublicationPhase::SynchronizeHead,
    CatalogPublicationPhase::VerifyHeadView,
    CatalogPublicationPhase::ReplaceHead,
    CatalogPublicationPhase::SynchronizeRoot,
];

/// Storage port that records calls and optionally fails at one exact phase.
pub struct RecordingStorage {
    observed: Vec<CatalogPublicationPhase>,
    failing_phase: Option<CatalogPublicationPhase>,
    readiness: CatalogPublicationReadiness,
}

impl RecordingStorage {
    /// Creates a recorder that admits every transition.
    pub const fn succeeding() -> Self {
        Self {
            observed: Vec::new(),
            failing_phase: None,
            readiness: CatalogPublicationReadiness::Ready,
        }
    }

    /// Creates a recorder that reports the complete candidate as current.
    pub const fn already_published() -> Self {
        Self {
            observed: Vec::new(),
            failing_phase: None,
            readiness: CatalogPublicationReadiness::AlreadyPublished,
        }
    }

    /// Creates a recorder that refuses one exact transition.
    pub const fn failing_at(phase: CatalogPublicationPhase) -> Self {
        Self {
            observed: Vec::new(),
            failing_phase: Some(phase),
            readiness: CatalogPublicationReadiness::Ready,
        }
    }

    /// Returns every transition attempted so far.
    pub fn observed(&self) -> &[CatalogPublicationPhase] {
        &self.observed
    }

    fn record(&mut self, phase: CatalogPublicationPhase) -> io::Result<()> {
        self.observed.push(phase);
        if self.failing_phase == Some(phase) {
            Err(io::Error::other("injected publication failure"))
        } else {
            Ok(())
        }
    }
}

impl CatalogPublicationStorage for RecordingStorage {
    fn verify_current(
        &mut self,
        _expected: CatalogPublicationExpectation,
        _candidate: &CatalogSnapshot<'_, '_, '_>,
        _segment: &SegmentPublication<'_, '_>,
    ) -> io::Result<CatalogPublicationReadiness> {
        self.record(CatalogPublicationPhase::VerifyCurrent)?;
        Ok(self.readiness)
    }

    fn link_segment(&mut self, _segment: &AdmittedSegment<'_>) -> io::Result<()> {
        self.record(CatalogPublicationPhase::LinkSegment)
    }

    fn verify_segment_pool(&mut self, _segment: &AdmittedSegment<'_>) -> io::Result<()> {
        self.record(CatalogPublicationPhase::VerifySegmentPool)
    }

    fn synchronize_segments(&mut self) -> io::Result<()> {
        self.record(CatalogPublicationPhase::SynchronizeSegments)
    }

    fn remove_segment_stage(&mut self) -> io::Result<()> {
        self.record(CatalogPublicationPhase::RemoveSegmentStage)
    }

    fn synchronize_staging_after_segment(&mut self) -> io::Result<()> {
        self.record(CatalogPublicationPhase::SynchronizeStagingAfterSegment)
    }

    fn create_catalog_stage(&mut self) -> io::Result<()> {
        self.record(CatalogPublicationPhase::CreateCatalogStage)
    }

    fn write_catalog(&mut self, _catalog: &CanonicalCatalog) -> io::Result<()> {
        self.record(CatalogPublicationPhase::WriteCatalog)
    }

    fn flush_catalog(&mut self) -> io::Result<()> {
        self.record(CatalogPublicationPhase::FlushCatalog)
    }

    fn synchronize_catalog(&mut self) -> io::Result<()> {
        self.record(CatalogPublicationPhase::SynchronizeCatalog)
    }

    fn link_catalog(&mut self, _catalog: ChecksummedCatalog<'_>) -> io::Result<()> {
        self.record(CatalogPublicationPhase::LinkCatalog)
    }

    fn verify_catalog_pool(&mut self, _catalog: ChecksummedCatalog<'_>) -> io::Result<()> {
        self.record(CatalogPublicationPhase::VerifyCatalogPool)
    }

    fn synchronize_catalogs(&mut self) -> io::Result<()> {
        self.record(CatalogPublicationPhase::SynchronizeCatalogs)
    }

    fn remove_catalog_stage(&mut self) -> io::Result<()> {
        self.record(CatalogPublicationPhase::RemoveCatalogStage)
    }

    fn synchronize_staging_after_catalog(&mut self) -> io::Result<()> {
        self.record(CatalogPublicationPhase::SynchronizeStagingAfterCatalog)
    }

    fn create_head_stage(&mut self) -> io::Result<()> {
        self.record(CatalogPublicationPhase::CreateHeadStage)
    }

    fn write_head(&mut self, _head: &CanonicalPublicationHead) -> io::Result<()> {
        self.record(CatalogPublicationPhase::WriteHead)
    }

    fn flush_head(&mut self) -> io::Result<()> {
        self.record(CatalogPublicationPhase::FlushHead)
    }

    fn synchronize_head(&mut self) -> io::Result<()> {
        self.record(CatalogPublicationPhase::SynchronizeHead)
    }

    fn verify_head_view(
        &mut self,
        _head: &CanonicalPublicationHead,
        _snapshot: &CatalogSnapshot<'_, '_, '_>,
    ) -> io::Result<()> {
        self.record(CatalogPublicationPhase::VerifyHeadView)
    }

    fn replace_head(&mut self) -> io::Result<()> {
        self.record(CatalogPublicationPhase::ReplaceHead)
    }

    fn synchronize_root(&mut self) -> io::Result<()> {
        self.record(CatalogPublicationPhase::SynchronizeRoot)
    }
}
