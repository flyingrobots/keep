//! This boundary module derives the canonical retirement intent from one
//! plan and the evidence execution observed, and owns the registered
//! derivations of the intent's proof, pool, and disposition-set digests.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use super::{
    CatalogSuccessorProofDigest, DispositionSetDigest, GcCandidate, GcCandidateSetDigest, GcPlan,
    GcRetirementIntent, GcRetirementIntentCoordinates, GcRetirementIntentError, PoolStateDigest,
    ReaderLockIdentity, SegmentPoolIdentityDigest,
};
use crate::adapters::SegmentDigest;
use crate::{
    CatalogDigest, CatalogGeneration, GcGeneration, LivenessGeneration, RegisteredRetentionProfile,
    RetentionManifestDigest,
};

const CATALOG_SUCCESSOR_PROOF_DOMAIN: &[u8] = b"keep.gc-catalog-successor-proof/v2\0";
const SEGMENT_POOL_DOMAIN: &[u8] = b"keep.gc-segment-pool/v2\0";
const DISPOSITION_SET_DOMAIN: &[u8] = b"keep.gc-disposition-set/v2\0";

/// The identity of one admitted segment pool: BLAKE3-256 under the
/// registered pool domain over the entry count and every `(digest, length)`
/// entry in canonical digest order.
#[must_use]
pub fn segment_pool_identity(inventory: &BTreeMap<SegmentDigest, u64>) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(SEGMENT_POOL_DOMAIN);
    hasher.update(
        &u32::try_from(inventory.len())
            .unwrap_or(u32::MAX)
            .to_be_bytes(),
    );
    for (digest, length) in inventory {
        hasher.update(digest.as_bytes());
        hasher.update(&length.to_be_bytes());
    }
    *hasher.finalize().as_bytes()
}

/// The identity of one admitted disposition set.
///
/// The registered empty-set digest for no receipts, otherwise BLAKE3-256
/// under the registered disposition-set domain over the count and every
/// receipt checksum in canonical order.
pub fn disposition_set_digest(checksums: &BTreeSet<[u8; 32]>) -> DispositionSetDigest {
    if checksums.is_empty() {
        return DispositionSetDigest::new(
            *crate::adapters::store_migration::empty_disposition_digest().as_bytes(),
        );
    }
    let mut hasher = blake3::Hasher::new();
    hasher.update(DISPOSITION_SET_DOMAIN);
    hasher.update(
        &u32::try_from(checksums.len())
            .unwrap_or(u32::MAX)
            .to_be_bytes(),
    );
    for checksum in checksums {
        hasher.update(checksum);
    }
    DispositionSetDigest::new(*hasher.finalize().as_bytes())
}

/// The proof that the catalog successor names no candidate: BLAKE3-256
/// under the registered proof domain over the catalog generation, catalog
/// digest, and the canonical candidate-set digest.
pub fn catalog_successor_proof(
    generation: CatalogGeneration,
    digest: CatalogDigest,
    candidate_set: GcCandidateSetDigest,
) -> CatalogSuccessorProofDigest {
    let mut hasher = blake3::Hasher::new();
    hasher.update(CATALOG_SUCCESSOR_PROOF_DOMAIN);
    hasher.update(&generation.get().to_be_bytes());
    hasher.update(digest.as_bytes());
    hasher.update(candidate_set.as_bytes());
    CatalogSuccessorProofDigest::new(*hasher.finalize().as_bytes())
}

/// What execution observed beyond the plan before deriving its intent.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GcIntentEvidence {
    /// The retirement generation: the successor of the last receipt's, or
    /// the initial generation.
    pub generation: GcGeneration,
    /// The exact retained realization profile.
    pub profile: RegisteredRetentionProfile,
    /// The exclusively locked `reader.lock`.
    pub reader_lock: ReaderLockIdentity,
}

/// Why a plan yields no intent.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GcIntentDerivationError {
    /// A candidate carries no release evidence in the snapshot.
    MissingEvidence {
        /// The candidate.
        segment: SegmentDigest,
    },
    /// The plan was computed under an empty retention state; retirement
    /// needs a published liveness generation to bind.
    EmptyRetention,
    /// The candidate set refused as an intent.
    Intent(GcRetirementIntentError),
}

impl fmt::Display for GcIntentDerivationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingEvidence { .. } => {
                formatter.write_str("a candidate carries no release evidence")
            }
            Self::EmptyRetention => {
                formatter.write_str("retirement needs a published retention head to bind")
            }
            Self::Intent(source) => write!(formatter, "{source}"),
        }
    }
}

impl std::error::Error for GcIntentDerivationError {}

/// Derives the one canonical intent for `plan` from the snapshot's release
/// evidence and `evidence` execution observed.
///
/// The intent's candidate set is the plan's candidates in canonical order,
/// each bound to the digest of the durable record that released it. The
/// catalog-successor proof, pool identity, and disposition-set digests are
/// the registered derivations above over the plan's own snapshot.
///
/// # Errors
///
/// Returns [`GcIntentDerivationError`] when a candidate lacks evidence, the
/// retention state is empty, or the candidate set refuses (an empty plan is
/// not an intent).
pub fn derive_gc_intent(
    plan: &GcPlan,
    snapshot: &super::GcLivenessSnapshot,
    evidence: GcIntentEvidence,
) -> Result<GcRetirementIntent, GcIntentDerivationError> {
    let coordinates = plan.coordinates();
    let (liveness_generation, manifest_digest) = match coordinates.retention() {
        super::GcRetentionState::Empty => return Err(GcIntentDerivationError::EmptyRetention),
        super::GcRetentionState::Published {
            generation,
            manifest_digest,
        } => (generation, manifest_digest),
    };
    let mut candidates = Vec::new();
    for candidate in plan.candidates() {
        let release = snapshot
            .release_evidence(candidate.segment())
            .ok_or_else(|| GcIntentDerivationError::MissingEvidence {
                segment: candidate.segment(),
            })?;
        candidates.push(GcCandidate::new(
            candidate.segment(),
            candidate.length(),
            release,
        ));
    }
    let candidate_set = super::intent_encoder::candidate_set_digest(&candidates);
    let intent_coordinates = GcRetirementIntentCoordinates {
        generation: evidence.generation,
        liveness_generation: bound_liveness(liveness_generation),
        manifest_digest: bound_manifest(manifest_digest),
        catalog_generation: coordinates.catalog_generation(),
        catalog_digest: coordinates.catalog_digest(),
        profile: evidence.profile,
        catalog_successor_proof_digest: catalog_successor_proof(
            coordinates.catalog_generation(),
            coordinates.catalog_digest(),
            candidate_set,
        ),
        segment_pool_identity_digest: SegmentPoolIdentityDigest::new(segment_pool_identity(
            snapshot.inventory(),
        )),
        disposition_set_digest: disposition_set_digest(snapshot.disposition_checksums()),
        reader_lock: evidence.reader_lock,
    };
    GcRetirementIntent::new(intent_coordinates, candidates).map_err(GcIntentDerivationError::Intent)
}

const fn bound_liveness(generation: LivenessGeneration) -> LivenessGeneration {
    generation
}

const fn bound_manifest(digest: RetentionManifestDigest) -> RetentionManifestDigest {
    digest
}

/// The pool state after every candidate is absent: the pool identity over
/// the inventory minus the candidates.
pub fn post_retirement_pool_state(
    inventory: &BTreeMap<SegmentDigest, u64>,
    retired: &[GcCandidate],
) -> PoolStateDigest {
    let remaining: BTreeMap<SegmentDigest, u64> = inventory
        .iter()
        .filter(|(digest, _)| !retired.iter().any(|c| c.segment_digest() == **digest))
        .map(|(digest, length)| (*digest, *length))
        .collect();
    PoolStateDigest::new(segment_pool_identity(&remaining))
}
