//! This boundary module owns storage-ready retention publication artifacts.

use super::{
    AdmittedRetentionRoot, CanonicalRetentionHead, CanonicalRetentionManifest,
    VerifiedRetentionClosure,
};
use crate::LivenessGeneration;

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
    candidate: AdmittedRetentionRoot<'encoded>,
    closure: VerifiedRetentionClosure,
    publication: Option<PreparedRetentionPublication>,
}

impl<'encoded> RetentionPublicationPreparation<'encoded> {
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
        candidate: AdmittedRetentionRoot<'encoded>,
        closure: VerifiedRetentionClosure,
        publication: PreparedRetentionPublication,
    ) -> Self {
        Self {
            candidate,
            closure,
            publication: Some(publication),
        }
    }

    pub(super) const fn already_committed(
        candidate: AdmittedRetentionRoot<'encoded>,
        closure: VerifiedRetentionClosure,
    ) -> Self {
        Self {
            candidate,
            closure,
            publication: None,
        }
    }
}
