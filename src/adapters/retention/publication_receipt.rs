//! This boundary module owns consequential retention publication receipts.

use super::{
    RetentionNamespaceAdmission, RetentionPublicationOutcome, RetentionPublicationPreparation,
};
use crate::{
    CatalogDigest, CatalogGeneration, LivenessGeneration, RegisteredRetentionProfile,
    RetentionAnchorSetDigest, RetentionClosureDigest, RetentionGenerationExpectation,
    RetentionManifestDigest, RetentionNamespaceDigest, RetentionRootDigest, RootGeneration,
};

/// Complete durable coordinates returned after retention publication.
#[must_use = "retention publication receipts bind the durable outcome"]
#[derive(Debug, Eq, PartialEq)]
pub struct RetentionPublicationReceipt {
    effect: RetentionPublicationEffect,
    namespace: RetentionNamespaceDigest,
    expected: RetentionGenerationExpectation,
    observed: Option<RootGeneration>,
    root_generation: RootGeneration,
    root_digest: RetentionRootDigest,
    liveness_generation: LivenessGeneration,
    manifest_digest: RetentionManifestDigest,
    profile: RegisteredRetentionProfile,
    anchor_set_digest: RetentionAnchorSetDigest,
    closure_digest: RetentionClosureDigest,
    catalog_generation: CatalogGeneration,
    catalog_digest: CatalogDigest,
}

impl RetentionPublicationReceipt {
    /// Returns the durable publication outcome.
    pub const fn outcome(&self) -> RetentionPublicationOutcome {
        match self.effect {
            RetentionPublicationEffect::Published(_) => RetentionPublicationOutcome::Published,
            RetentionPublicationEffect::AlreadyCommitted => {
                RetentionPublicationOutcome::AlreadyCommitted
            }
        }
    }

    /// Returns namespace creation or admission for a new publication.
    pub const fn namespace_admission(&self) -> Option<RetentionNamespaceAdmission> {
        match self.effect {
            RetentionPublicationEffect::Published(admission) => Some(admission),
            RetentionPublicationEffect::AlreadyCommitted => None,
        }
    }

    /// Returns the selected retention namespace digest.
    pub const fn namespace(&self) -> RetentionNamespaceDigest {
        self.namespace
    }

    /// Returns the caller-supplied expected namespace generation.
    pub const fn expected(&self) -> RetentionGenerationExpectation {
        self.expected
    }

    /// Returns the namespace generation observed before publication.
    pub const fn observed(&self) -> Option<RootGeneration> {
        self.observed
    }

    /// Returns the committed namespace root generation.
    pub const fn root_generation(&self) -> RootGeneration {
        self.root_generation
    }

    /// Returns the committed canonical root digest.
    pub const fn root_digest(&self) -> RetentionRootDigest {
        self.root_digest
    }

    /// Returns the selected global liveness generation.
    pub const fn liveness_generation(&self) -> LivenessGeneration {
        self.liveness_generation
    }

    /// Returns the selected global manifest digest.
    pub const fn manifest_digest(&self) -> RetentionManifestDigest {
        self.manifest_digest
    }

    /// Returns the registered realization profile.
    pub const fn profile(&self) -> RegisteredRetentionProfile {
        self.profile
    }

    /// Returns the verified anchor-set digest.
    pub const fn anchor_set_digest(&self) -> RetentionAnchorSetDigest {
        self.anchor_set_digest
    }

    /// Returns the verified closure transcript digest.
    pub const fn closure_digest(&self) -> RetentionClosureDigest {
        self.closure_digest
    }

    /// Returns the pinned catalog generation.
    pub const fn catalog_generation(&self) -> CatalogGeneration {
        self.catalog_generation
    }

    /// Returns the pinned catalog digest.
    pub const fn catalog_digest(&self) -> CatalogDigest {
        self.catalog_digest
    }

    pub(super) fn published(
        namespace_admission: RetentionNamespaceAdmission,
        preparation: &RetentionPublicationPreparation<'_>,
    ) -> Self {
        Self::new(
            RetentionPublicationEffect::Published(namespace_admission),
            preparation,
        )
    }

    pub(super) fn already_committed(preparation: &RetentionPublicationPreparation<'_>) -> Self {
        Self::new(RetentionPublicationEffect::AlreadyCommitted, preparation)
    }

    fn new(
        effect: RetentionPublicationEffect,
        preparation: &RetentionPublicationPreparation<'_>,
    ) -> Self {
        let candidate = preparation.candidate();
        let closure = preparation.closure();
        Self {
            effect,
            namespace: candidate.root().namespace().digest(),
            expected: preparation.expected(),
            observed: preparation.observed(),
            root_generation: candidate.root().generation(),
            root_digest: candidate.digest(),
            liveness_generation: preparation.liveness_generation(),
            manifest_digest: preparation.manifest_digest(),
            profile: candidate.root().profile(),
            anchor_set_digest: candidate.anchor_set_digest(),
            closure_digest: closure.digest(),
            catalog_generation: closure.catalog_generation(),
            catalog_digest: closure.catalog_digest(),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
enum RetentionPublicationEffect {
    Published(RetentionNamespaceAdmission),
    AlreadyCommitted,
}
