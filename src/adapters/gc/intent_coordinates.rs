//! This boundary module owns the liveness, catalog, profile, pool, and
//! reader-lock coordinates one GC retirement intent binds.

use super::{
    CatalogSuccessorProofDigest, DispositionSetDigest, ReaderLockIdentity,
    SegmentPoolIdentityDigest,
};
use crate::{
    CatalogDigest, CatalogGeneration, GcGeneration, LivenessGeneration, RegisteredRetentionProfile,
    RetentionManifestDigest,
};

/// Every coordinate a retirement intent binds besides its candidates.
///
/// Fields are public because each is an already-validated typed value; the
/// struct only groups them so an intent is constructed from one value.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GcRetirementIntentCoordinates {
    /// Generation of this retirement.
    pub generation: GcGeneration,
    /// Exact current global liveness generation.
    pub liveness_generation: LivenessGeneration,
    /// Exact current retention-manifest digest.
    pub manifest_digest: RetentionManifestDigest,
    /// Exact catalog successor generation that names no candidate.
    pub catalog_generation: CatalogGeneration,
    /// Exact catalog successor digest.
    pub catalog_digest: CatalogDigest,
    /// Exact retained realization profile.
    pub profile: RegisteredRetentionProfile,
    /// Digest of the complete catalog-successor proof.
    pub catalog_successor_proof_digest: CatalogSuccessorProofDigest,
    /// Identity digest of the exact admitted segment pool.
    pub segment_pool_identity_digest: SegmentPoolIdentityDigest,
    /// Digest of the exact admitted disposition receipts.
    pub disposition_set_digest: DispositionSetDigest,
    /// Identity of the exclusively locked `reader.lock`.
    pub reader_lock: ReaderLockIdentity,
}
