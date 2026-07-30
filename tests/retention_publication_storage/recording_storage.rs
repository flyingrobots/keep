//! Deterministic retention publication storage recorder.

use std::io;

use keep::{
    AdmittedRetentionRoot, CanonicalRetentionHead, CanonicalRetentionManifest,
    RetentionNamespaceAdmission, RetentionPublicationPhase, RetentionPublicationStorage,
};

/// Storage port that records every attempted publication phase.
#[derive(Default)]
pub struct RecordingStorage {
    observed: Vec<RetentionPublicationPhase>,
}

impl RecordingStorage {
    /// Creates an empty recorder.
    pub const fn new() -> Self {
        Self {
            observed: Vec::new(),
        }
    }

    /// Returns every recorded phase in call order.
    pub fn observed(&self) -> &[RetentionPublicationPhase] {
        &self.observed
    }

    fn record(&mut self, phase: RetentionPublicationPhase) {
        self.observed.push(phase);
    }
}

impl RetentionPublicationStorage for RecordingStorage {
    fn write_root_stage(&mut self, _root: &AdmittedRetentionRoot<'_>) -> io::Result<()> {
        self.record(RetentionPublicationPhase::WriteRootStage);
        Ok(())
    }

    fn synchronize_root_stage(&mut self) -> io::Result<()> {
        self.record(RetentionPublicationPhase::SynchronizeRootStage);
        Ok(())
    }

    fn admit_root_namespace(
        &mut self,
        _root: &AdmittedRetentionRoot<'_>,
    ) -> io::Result<RetentionNamespaceAdmission> {
        self.record(RetentionPublicationPhase::AdmitRootNamespace);
        Ok(RetentionNamespaceAdmission::Created)
    }

    fn synchronize_roots_after_namespace(&mut self) -> io::Result<()> {
        self.record(RetentionPublicationPhase::SynchronizeRootsAfterNamespace);
        Ok(())
    }

    fn link_root(&mut self, _root: &AdmittedRetentionRoot<'_>) -> io::Result<()> {
        self.record(RetentionPublicationPhase::LinkRoot);
        Ok(())
    }

    fn synchronize_root_namespace(&mut self, _root: &AdmittedRetentionRoot<'_>) -> io::Result<()> {
        self.record(RetentionPublicationPhase::SynchronizeRootNamespace);
        Ok(())
    }

    fn write_manifest_stage(&mut self, _manifest: &CanonicalRetentionManifest) -> io::Result<()> {
        self.record(RetentionPublicationPhase::WriteManifestStage);
        Ok(())
    }

    fn synchronize_manifest_stage(&mut self) -> io::Result<()> {
        self.record(RetentionPublicationPhase::SynchronizeManifestStage);
        Ok(())
    }

    fn link_manifest(&mut self, _manifest: &CanonicalRetentionManifest) -> io::Result<()> {
        self.record(RetentionPublicationPhase::LinkManifest);
        Ok(())
    }

    fn synchronize_manifest_pool(&mut self) -> io::Result<()> {
        self.record(RetentionPublicationPhase::SynchronizeManifestPool);
        Ok(())
    }

    fn write_head_stage(&mut self, _head: &CanonicalRetentionHead) -> io::Result<()> {
        self.record(RetentionPublicationPhase::WriteHeadStage);
        Ok(())
    }

    fn synchronize_head_stage(&mut self) -> io::Result<()> {
        self.record(RetentionPublicationPhase::SynchronizeHeadStage);
        Ok(())
    }

    fn replace_head(&mut self) -> io::Result<()> {
        self.record(RetentionPublicationPhase::ReplaceHead);
        Ok(())
    }

    fn synchronize_retention_namespace(&mut self) -> io::Result<()> {
        self.record(RetentionPublicationPhase::SynchronizeRetentionNamespace);
        Ok(())
    }

    fn remove_root_stage(&mut self) -> io::Result<()> {
        self.record(RetentionPublicationPhase::RemoveRootStage);
        Ok(())
    }

    fn remove_manifest_stage(&mut self) -> io::Result<()> {
        self.record(RetentionPublicationPhase::RemoveManifestStage);
        Ok(())
    }

    fn synchronize_cleanup(&mut self) -> io::Result<()> {
        self.record(RetentionPublicationPhase::SynchronizeCleanup);
        Ok(())
    }
}
