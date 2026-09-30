//! This boundary module owns the semantic recovery-disposition receipt.

use super::{
    ArtifactContentDigest, ArtifactIdentityDigest, DecisionEvidenceDigest, ObservedHeadChecksum,
    ReaderLockIdentity, RecoveryArtifactKind, RecoveryClassification, RecoveryDispositionDecision,
};
use crate::{CatalogDigest, CatalogGeneration, LivenessGeneration, RetentionManifestDigest};

/// The coordinates one disposition was decided under.
///
/// Every field is observed by the writer under writer authority and the
/// exclusive reader lock; the receipt transports them so a later planner
/// can refuse a disposition whose coordinates no longer hold.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RecoveryDispositionCoordinates {
    /// The publication-head generation observed.
    pub publication_generation: CatalogGeneration,
    /// The publication-head checksum observed.
    pub publication_checksum: ObservedHeadChecksum,
    /// The catalog generation observed.
    pub catalog_generation: CatalogGeneration,
    /// The catalog digest observed.
    pub catalog_digest: CatalogDigest,
    /// The liveness generation observed.
    pub liveness_generation: LivenessGeneration,
    /// The retention-manifest digest observed.
    pub manifest_digest: RetentionManifestDigest,
    /// The exclusively locked `reader.lock`.
    pub reader_lock: ReaderLockIdentity,
}

/// The artifact one disposition names.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RecoveryDispositionArtifact {
    /// The artifact kind.
    pub kind: RecoveryArtifactKind,
    /// The classification the artifact was admitted under.
    pub classification: RecoveryClassification,
    /// The exact observed length.
    pub length: u64,
    /// The physical evidence identity: the pool name digest.
    pub identity_digest: ArtifactIdentityDigest,
    /// The digest of the exact verified bytes under the artifact domain.
    pub content_digest: ArtifactContentDigest,
}

/// One explicit finalize-or-retire decision over one recovery-protected
/// artifact.
///
/// A receipt is a statement about the decision and the coordinates it was
/// made under. It does not prove the artifact was finalized or unlinked;
/// the disposition protocol establishes that separately.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RecoveryDispositionReceipt {
    artifact: RecoveryDispositionArtifact,
    decision: RecoveryDispositionDecision,
    coordinates: RecoveryDispositionCoordinates,
    evidence_digest: DecisionEvidenceDigest,
}

impl RecoveryDispositionReceipt {
    /// Binds one decision to one artifact under one set of coordinates.
    pub const fn new(
        artifact: RecoveryDispositionArtifact,
        decision: RecoveryDispositionDecision,
        coordinates: RecoveryDispositionCoordinates,
        evidence_digest: DecisionEvidenceDigest,
    ) -> Self {
        Self {
            artifact,
            decision,
            coordinates,
            evidence_digest,
        }
    }

    /// Returns the disposed artifact.
    #[must_use]
    pub const fn artifact(&self) -> RecoveryDispositionArtifact {
        self.artifact
    }

    /// Returns the decision.
    #[must_use]
    pub const fn decision(&self) -> RecoveryDispositionDecision {
        self.decision
    }

    /// Returns the coordinates the decision was made under.
    #[must_use]
    pub const fn coordinates(&self) -> RecoveryDispositionCoordinates {
        self.coordinates
    }

    /// Returns the digest of the complete decision evidence.
    pub const fn evidence_digest(&self) -> DecisionEvidenceDigest {
        self.evidence_digest
    }
}
