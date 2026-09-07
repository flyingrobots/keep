//! This module owns the state of one admitted retention publication attempt.

use std::io;

use cap_std::fs::Dir;

use super::CanonicalRetentionManifest;
use super::filesystem_retention_pool_name as pool_name;
use super::filesystem_retention_stage::{FilesystemRetentionStage, invalid_data};
use crate::{LivenessGeneration, RetentionGenerationExpectation};

/// Everything one publication attempt retains between storage-port phases.
///
/// An attempt exists only after current-state verification admits a forward
/// publication, and it is discarded when the next verification begins or
/// cleanup completes. No stage handle, pool coordinate, namespace capability,
/// or expectation from a refused or interrupted run can therefore reach a
/// later phase.
pub(super) struct PublicationAttempt {
    expected: RetentionGenerationExpectation,
    liveness_generation: LivenessGeneration,
    namespace: Option<Dir>,
    retained_root: Option<String>,
    retained_manifest: Option<String>,
    root_stage: Option<FilesystemRetentionStage>,
    manifest_stage: Option<FilesystemRetentionStage>,
    head_stage: Option<FilesystemRetentionStage>,
}

/// Returns the admitted attempt or refuses because no verification admitted one.
pub(super) fn require(attempt: Option<&PublicationAttempt>) -> io::Result<&PublicationAttempt> {
    attempt.ok_or_else(no_attempt)
}

/// Mutable form of [`require`] for phases that retain state on the attempt.
pub(super) fn require_mut(
    attempt: &mut Option<PublicationAttempt>,
) -> io::Result<&mut PublicationAttempt> {
    attempt.as_mut().ok_or_else(no_attempt)
}

fn no_attempt() -> io::Error {
    invalid_data("no admitted retention publication attempt")
}

impl PublicationAttempt {
    pub(super) const fn new(
        expected: RetentionGenerationExpectation,
        liveness_generation: LivenessGeneration,
    ) -> Self {
        Self {
            expected,
            liveness_generation,
            namespace: None,
            retained_root: None,
            retained_manifest: None,
            root_stage: None,
            manifest_stage: None,
            head_stage: None,
        }
    }

    pub(super) const fn expected(&self) -> RetentionGenerationExpectation {
        self.expected
    }

    /// Names the manifest pool entry at this attempt's liveness generation.
    pub(super) fn manifest_name(&self, manifest: &CanonicalRetentionManifest) -> String {
        pool_name::manifest(self.liveness_generation, manifest.digest())
    }

    pub(super) fn retain_namespace(&mut self, namespace: Dir) {
        self.namespace = Some(namespace);
    }

    pub(super) fn namespace(&self) -> io::Result<&Dir> {
        self.namespace
            .as_ref()
            .ok_or_else(|| invalid_data("retention root namespace was not admitted"))
    }

    pub(super) fn retain_root_name(&mut self, name: String) {
        self.retained_root = Some(name);
    }

    pub(super) fn retained_root_name(&self) -> io::Result<&str> {
        self.retained_root
            .as_deref()
            .ok_or_else(|| invalid_data("retention root pool coordinate was not retained"))
    }

    pub(super) fn retain_manifest_name(&mut self, name: String) {
        self.retained_manifest = Some(name);
    }

    pub(super) fn retained_manifest_name(&self) -> io::Result<&str> {
        self.retained_manifest
            .as_deref()
            .ok_or_else(|| invalid_data("retention manifest pool coordinate was not retained"))
    }

    pub(super) fn retain_root_stage(&mut self, stage: FilesystemRetentionStage) {
        self.root_stage = Some(stage);
    }

    pub(super) fn root_stage(&self) -> io::Result<&FilesystemRetentionStage> {
        self.root_stage
            .as_ref()
            .ok_or_else(|| invalid_data("retention root stage was not retained"))
    }

    pub(super) fn take_root_stage(&mut self) -> io::Result<FilesystemRetentionStage> {
        self.root_stage
            .take()
            .ok_or_else(|| invalid_data("retention root stage was not retained"))
    }

    pub(super) fn retain_manifest_stage(&mut self, stage: FilesystemRetentionStage) {
        self.manifest_stage = Some(stage);
    }

    pub(super) fn manifest_stage(&self) -> io::Result<&FilesystemRetentionStage> {
        self.manifest_stage
            .as_ref()
            .ok_or_else(|| invalid_data("retention manifest stage was not retained"))
    }

    pub(super) fn take_manifest_stage(&mut self) -> io::Result<FilesystemRetentionStage> {
        self.manifest_stage
            .take()
            .ok_or_else(|| invalid_data("retention manifest stage was not retained"))
    }

    pub(super) fn retain_head_stage(&mut self, stage: FilesystemRetentionStage) {
        self.head_stage = Some(stage);
    }

    pub(super) fn head_stage(&self) -> io::Result<&FilesystemRetentionStage> {
        self.head_stage
            .as_ref()
            .ok_or_else(|| invalid_data("retention head stage was not retained"))
    }

    pub(super) fn take_head_stage(&mut self) -> io::Result<FilesystemRetentionStage> {
        self.head_stage
            .take()
            .ok_or_else(|| invalid_data("retention head stage was not retained"))
    }
}
