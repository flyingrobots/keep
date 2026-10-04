//! Deterministic retention publication storage recorder.

use std::io;

use keep::{
    AdmittedRetentionRoot, CanonicalRetentionHead, CanonicalRetentionManifest,
    RetentionNamespaceAdmission, RetentionPublicationPhase, RetentionPublicationPreparation,
    RetentionPublicationStorage, RetentionTransitionDisposition,
};

/// Storage port that records every attempted publication phase.
pub struct RecordingStorage {
    observed: Vec<RetentionPublicationPhase>,
    verification_count: usize,
    disposition: RetentionTransitionDisposition,
    namespace_admission: RetentionNamespaceAdmission,
    fail_at: Option<RetentionPublicationPhase>,
    verification_failure: Option<io::ErrorKind>,
}

impl Default for RecordingStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl RecordingStorage {
    /// Creates an empty recorder.
    pub const fn new() -> Self {
        Self {
            observed: Vec::new(),
            verification_count: 0,
            disposition: RetentionTransitionDisposition::Publish,
            namespace_admission: RetentionNamespaceAdmission::Created,
            fail_at: None,
            verification_failure: None,
        }
    }

    /// Creates a recorder that observes the candidate as already committed.
    pub const fn already_committed() -> Self {
        Self {
            observed: Vec::new(),
            verification_count: 0,
            disposition: RetentionTransitionDisposition::AlreadyCommitted,
            namespace_admission: RetentionNamespaceAdmission::Created,
            fail_at: None,
            verification_failure: None,
        }
    }

    /// Creates a publisher that admits an existing root namespace.
    pub const fn existing_namespace() -> Self {
        Self {
            observed: Vec::new(),
            verification_count: 0,
            disposition: RetentionTransitionDisposition::Publish,
            namespace_admission: RetentionNamespaceAdmission::Existing,
            fail_at: None,
            verification_failure: None,
        }
    }

    /// Creates a publisher that fails at one exact durability phase.
    pub const fn failing_at(phase: RetentionPublicationPhase) -> Self {
        Self {
            observed: Vec::new(),
            verification_count: 0,
            disposition: RetentionTransitionDisposition::Publish,
            namespace_admission: RetentionNamespaceAdmission::Created,
            fail_at: Some(phase),
            verification_failure: None,
        }
    }

    /// Creates a publisher that refuses current-authority verification.
    pub const fn verification_failure() -> Self {
        Self {
            observed: Vec::new(),
            verification_count: 0,
            disposition: RetentionTransitionDisposition::Publish,
            namespace_admission: RetentionNamespaceAdmission::Created,
            fail_at: None,
            verification_failure: Some(io::ErrorKind::PermissionDenied),
        }
    }

    /// Returns every recorded phase in call order.
    pub fn observed(&self) -> &[RetentionPublicationPhase] {
        &self.observed
    }

    /// Returns the number of authority-verification calls.
    pub const fn verification_count(&self) -> usize {
        self.verification_count
    }

    fn record(&mut self, phase: RetentionPublicationPhase) -> io::Result<()> {
        self.observed.push(phase);
        if self.fail_at == Some(phase) {
            Err(io::Error::other("injected retention publication failure"))
        } else {
            Ok(())
        }
    }
}

impl RetentionPublicationStorage for RecordingStorage {
    fn verify_current(
        &mut self,
        _preparation: &RetentionPublicationPreparation<'_>,
    ) -> io::Result<RetentionTransitionDisposition> {
        self.verification_count = self
            .verification_count
            .checked_add(1)
            .ok_or_else(|| io::Error::other("verification count overflow"))?;
        match self.verification_failure {
            Some(kind) => Err(io::Error::new(kind, "injected authority failure")),
            None => Ok(self.disposition),
        }
    }

    fn write_root_stage(&mut self, _root: &AdmittedRetentionRoot<'_>) -> io::Result<()> {
        self.record(RetentionPublicationPhase::WriteRootStage)
    }

    fn synchronize_root_stage(&mut self) -> io::Result<()> {
        self.record(RetentionPublicationPhase::SynchronizeRootStage)
    }

    fn admit_root_namespace(
        &mut self,
        _root: &AdmittedRetentionRoot<'_>,
    ) -> io::Result<RetentionNamespaceAdmission> {
        self.record(RetentionPublicationPhase::AdmitRootNamespace)?;
        Ok(self.namespace_admission)
    }

    fn synchronize_roots_after_namespace(&mut self) -> io::Result<()> {
        self.record(RetentionPublicationPhase::SynchronizeRootsAfterNamespace)
    }

    fn link_root(&mut self, _root: &AdmittedRetentionRoot<'_>) -> io::Result<()> {
        self.record(RetentionPublicationPhase::LinkRoot)
    }

    fn synchronize_root_namespace(&mut self, _root: &AdmittedRetentionRoot<'_>) -> io::Result<()> {
        self.record(RetentionPublicationPhase::SynchronizeRootNamespace)
    }

    fn write_manifest_stage(&mut self, _manifest: &CanonicalRetentionManifest) -> io::Result<()> {
        self.record(RetentionPublicationPhase::WriteManifestStage)
    }

    fn synchronize_manifest_stage(&mut self) -> io::Result<()> {
        self.record(RetentionPublicationPhase::SynchronizeManifestStage)
    }

    fn link_manifest(&mut self, _manifest: &CanonicalRetentionManifest) -> io::Result<()> {
        self.record(RetentionPublicationPhase::LinkManifest)
    }

    fn synchronize_manifest_pool(&mut self) -> io::Result<()> {
        self.record(RetentionPublicationPhase::SynchronizeManifestPool)
    }

    fn write_head_stage(&mut self, _head: &CanonicalRetentionHead) -> io::Result<()> {
        self.record(RetentionPublicationPhase::WriteHeadStage)
    }

    fn synchronize_head_stage(&mut self) -> io::Result<()> {
        self.record(RetentionPublicationPhase::SynchronizeHeadStage)
    }

    fn replace_head(&mut self) -> io::Result<()> {
        self.record(RetentionPublicationPhase::ReplaceHead)
    }

    fn synchronize_retention_namespace(&mut self) -> io::Result<()> {
        self.record(RetentionPublicationPhase::SynchronizeRetentionNamespace)
    }

    fn remove_root_stage(&mut self) -> io::Result<()> {
        self.record(RetentionPublicationPhase::RemoveRootStage)
    }

    fn remove_manifest_stage(&mut self) -> io::Result<()> {
        self.record(RetentionPublicationPhase::RemoveManifestStage)
    }

    fn synchronize_cleanup(&mut self) -> io::Result<()> {
        self.record(RetentionPublicationPhase::SynchronizeCleanup)
    }
}
