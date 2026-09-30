//! This boundary module owns post-integrity GC retirement intent header
//! admission.

use super::GcRetirementIntentDecodeError as Error;
use super::intent_header_decoder::DecodedIntentHeader;
use super::{
    CatalogSuccessorProofDigest, DispositionSetDigest, GcRetirementIntent,
    GcRetirementIntentCoordinates, ReaderLockIdentity, SegmentPoolIdentityDigest,
};
use crate::{
    CatalogDigest, CatalogGeneration, GcGeneration, LivenessGeneration, RegisteredRetentionProfile,
    RetentionManifestDigest,
};

pub(super) fn admit(header: &DecodedIntentHeader) -> Result<GcRetirementIntentCoordinates, Error> {
    if header.candidate_count > GcRetirementIntent::MAXIMUM_CANDIDATE_COUNT {
        return Err(Error::CandidateCountExceeded {
            maximum: GcRetirementIntent::MAXIMUM_CANDIDATE_COUNT,
            observed: header.candidate_count,
        });
    }
    let generation =
        GcGeneration::new(header.generation).map_err(|source| Error::Generation { source })?;
    let liveness_generation = LivenessGeneration::new(header.liveness_generation)
        .map_err(|source| Error::LivenessGeneration { source })?;
    let catalog_generation = CatalogGeneration::new(header.catalog_generation)
        .map_err(|source| Error::CatalogGeneration { source })?;
    let profile = RegisteredRetentionProfile::admit(
        header.profile_identity,
        header.profile_version,
        header.profile_digest,
    )
    .map_err(|source| Error::Profile { source })?;
    Ok(GcRetirementIntentCoordinates {
        generation,
        liveness_generation,
        manifest_digest: RetentionManifestDigest::from_hash(header.manifest_digest),
        catalog_generation,
        catalog_digest: CatalogDigest::from_validated(header.catalog_digest),
        profile,
        catalog_successor_proof_digest: CatalogSuccessorProofDigest::new(
            header.catalog_successor_proof_digest,
        ),
        segment_pool_identity_digest: SegmentPoolIdentityDigest::new(
            header.segment_pool_identity_digest,
        ),
        disposition_set_digest: DispositionSetDigest::new(header.disposition_set_digest),
        reader_lock: ReaderLockIdentity::new(
            header.reader_device,
            header.reader_mount,
            header.reader_file,
        ),
    })
}
