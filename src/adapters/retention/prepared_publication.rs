//! This boundary module owns storage-ready retention publication artifacts.

use super::{
    AdmittedRetentionRoot, CanonicalRetentionHead, CanonicalRetentionManifest,
    RetentionTransitionDisposition, VerifiedRetentionClosure,
};
use crate::{LivenessGeneration, RetentionGenerationExpectation, RootGeneration};

/// Canonical global artifacts ready for ordered storage execution.
#[must_use = "prepared retention publication must be executed or handled explicitly"]
#[derive(Debug)]
pub struct PreparedRetentionPublication {
    manifest: CanonicalRetentionManifest,
    head: CanonicalRetentionHead,
    liveness_generation: LivenessGeneration,
}

impl PreparedRetentionPublication {
    /// Returns the complete canonical successor manifest.
    pub const fn manifest(&self) -> &CanonicalRetentionManifest {
        &self.manifest
    }

    /// Returns the complete canonical successor head.
    pub const fn head(&self) -> &CanonicalRetentionHead {
        &self.head
    }

    /// Returns the exact successor global liveness generation.
    pub const fn liveness_generation(&self) -> LivenessGeneration {
        self.liveness_generation
    }

    pub(super) const fn new(
        manifest: CanonicalRetentionManifest,
        head: CanonicalRetentionHead,
        liveness_generation: LivenessGeneration,
    ) -> Self {
        Self {
            manifest,
            head,
            liveness_generation,
        }
    }
}

/// Result of binding one preflight proof to the current global manifest.
#[must_use = "retention publication preparation must be handled explicitly"]
#[derive(Debug)]
pub struct RetentionPublicationPreparation<'encoded> {
    disposition: RetentionTransitionDisposition,
    expected: RetentionGenerationExpectation,
    observed: Option<RootGeneration>,
    candidate: AdmittedRetentionRoot<'encoded>,
    closure: VerifiedRetentionClosure,
    publication: Option<PreparedRetentionPublication>,
}

impl<'encoded> RetentionPublicationPreparation<'encoded> {
    /// Returns whether the candidate requires publication or is current.
    pub const fn disposition(&self) -> RetentionTransitionDisposition {
        self.disposition
    }

    /// Returns the caller-supplied expected namespace generation.
    pub const fn expected(&self) -> RetentionGenerationExpectation {
        self.expected
    }

    /// Returns the namespace generation observed during transition planning.
    pub const fn observed(&self) -> Option<RootGeneration> {
        self.observed
    }

    /// Borrows the admitted candidate root.
    pub const fn candidate(&self) -> &AdmittedRetentionRoot<'encoded> {
        &self.candidate
    }

    /// Returns the revalidated candidate closure.
    pub const fn closure(&self) -> VerifiedRetentionClosure {
        self.closure
    }

    /// Returns new global artifacts, or normal absence for an exact retry.
    pub const fn publication(&self) -> Option<&PreparedRetentionPublication> {
        self.publication.as_ref()
    }

    pub(super) const fn publish(
        expected: RetentionGenerationExpectation,
        observed: Option<RootGeneration>,
        candidate: AdmittedRetentionRoot<'encoded>,
        closure: VerifiedRetentionClosure,
        publication: PreparedRetentionPublication,
    ) -> Self {
        Self {
            disposition: RetentionTransitionDisposition::Publish,
            expected,
            observed,
            candidate,
            closure,
            publication: Some(publication),
        }
    }

    pub(super) const fn already_committed(
        expected: RetentionGenerationExpectation,
        observed: Option<RootGeneration>,
        candidate: AdmittedRetentionRoot<'encoded>,
        closure: VerifiedRetentionClosure,
    ) -> Self {
        Self {
            disposition: RetentionTransitionDisposition::AlreadyCommitted,
            expected,
            observed,
            candidate,
            closure,
            publication: None,
        }
    }
}
