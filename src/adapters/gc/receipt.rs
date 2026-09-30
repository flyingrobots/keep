//! This boundary module owns the semantic GC retirement receipt.

use super::{
    GcCandidateSetDigest, GcRetirementIntent, GcRetirementIntentDigest, PoolStateDigest,
    ReaderLockIdentity,
};
use crate::{
    CatalogDigest, CatalogGeneration, GcGeneration, LivenessGeneration, RetentionManifestDigest,
};

/// The completion statement of one retirement.
///
/// Every coordinate except the pool-state digest and the synchronization
/// count is bound from the intent the receipt completes; the executor
/// revalidates them and the decoder refuses a receipt that disagrees with
/// its intent. The synchronization count is one per unlinked candidate.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GcRetirementReceipt {
    generation: GcGeneration,
    intent_digest: GcRetirementIntentDigest,
    retired_candidate_set_digest: GcCandidateSetDigest,
    pool_state_digest: PoolStateDigest,
    liveness_generation: LivenessGeneration,
    manifest_digest: RetentionManifestDigest,
    catalog_generation: CatalogGeneration,
    catalog_digest: CatalogDigest,
    reader_lock: ReaderLockIdentity,
    synchronization_count: u64,
}

/// Every field of one receipt read without an intent to bind it to.
#[derive(Clone, Copy)]
pub(super) struct GcRetirementReceiptFields {
    pub(super) generation: GcGeneration,
    pub(super) intent_digest: GcRetirementIntentDigest,
    pub(super) retired_candidate_set_digest: GcCandidateSetDigest,
    pub(super) pool_state_digest: PoolStateDigest,
    pub(super) liveness_generation: LivenessGeneration,
    pub(super) manifest_digest: RetentionManifestDigest,
    pub(super) catalog_generation: CatalogGeneration,
    pub(super) catalog_digest: CatalogDigest,
    pub(super) reader_lock: ReaderLockIdentity,
    pub(super) synchronization_count: u64,
}

impl GcRetirementReceipt {
    pub(super) const fn from_fields(fields: GcRetirementReceiptFields) -> Self {
        Self {
            generation: fields.generation,
            intent_digest: fields.intent_digest,
            retired_candidate_set_digest: fields.retired_candidate_set_digest,
            pool_state_digest: fields.pool_state_digest,
            liveness_generation: fields.liveness_generation,
            manifest_digest: fields.manifest_digest,
            catalog_generation: fields.catalog_generation,
            catalog_digest: fields.catalog_digest,
            reader_lock: fields.reader_lock,
            synchronization_count: fields.synchronization_count,
        }
    }

    pub(super) fn for_intent(
        intent_digest: GcRetirementIntentDigest,
        retired_candidate_set_digest: GcCandidateSetDigest,
        intent: &GcRetirementIntent,
        pool_state_digest: PoolStateDigest,
    ) -> Self {
        let coordinates = intent.coordinates();
        Self {
            generation: coordinates.generation,
            intent_digest,
            retired_candidate_set_digest,
            pool_state_digest,
            liveness_generation: coordinates.liveness_generation,
            manifest_digest: coordinates.manifest_digest,
            catalog_generation: coordinates.catalog_generation,
            catalog_digest: coordinates.catalog_digest,
            reader_lock: coordinates.reader_lock,
            synchronization_count: u64::from(intent.candidate_count()),
        }
    }

    /// Returns the completed retirement generation.
    pub const fn generation(&self) -> GcGeneration {
        self.generation
    }

    /// Returns the digest of the completed intent.
    pub const fn intent_digest(&self) -> GcRetirementIntentDigest {
        self.intent_digest
    }

    /// Returns the digest of the retired candidate set.
    pub const fn retired_candidate_set_digest(&self) -> GcCandidateSetDigest {
        self.retired_candidate_set_digest
    }

    /// Returns the digest of the synchronized pool after retirement.
    pub const fn pool_state_digest(&self) -> PoolStateDigest {
        self.pool_state_digest
    }

    /// Returns the revalidated liveness generation.
    pub const fn liveness_generation(&self) -> LivenessGeneration {
        self.liveness_generation
    }

    /// Returns the revalidated retention-manifest digest.
    pub const fn manifest_digest(&self) -> RetentionManifestDigest {
        self.manifest_digest
    }

    /// Returns the revalidated catalog generation.
    pub const fn catalog_generation(&self) -> CatalogGeneration {
        self.catalog_generation
    }

    /// Returns the revalidated catalog digest.
    pub const fn catalog_digest(&self) -> CatalogDigest {
        self.catalog_digest
    }

    /// Returns the identity of the exclusively locked `reader.lock`.
    pub const fn reader_lock(&self) -> ReaderLockIdentity {
        self.reader_lock
    }

    /// Returns the number of completed pool-directory synchronizations.
    #[must_use]
    pub const fn synchronization_count(&self) -> u64 {
        self.synchronization_count
    }
}
