//! This boundary module owns canonical GC retirement intent encoding.

use super::{
    CanonicalGcRetirementIntent, GcCandidate, GcCandidateSetDigest, GcRetirementIntent,
    GcRetirementIntentCoordinates, GcRetirementIntentDigest, GcRetirementIntentEncodeError,
    intent_format as format,
};

pub(super) fn encode(
    intent: &GcRetirementIntent,
) -> Result<CanonicalGcRetirementIntent, GcRetirementIntentEncodeError> {
    let candidate_count = intent.candidate_count();
    let total_length = format::canonical_length(candidate_count)
        .ok_or(GcRetirementIntentEncodeError::LengthOverflow)?;
    let declared_length =
        u64::try_from(total_length).map_err(|_| GcRetirementIntentEncodeError::LengthOverflow)?;
    let mut candidates = Vec::new();
    candidates
        .try_reserve_exact(
            total_length
                .checked_sub(format::HEADER_LENGTH)
                .and_then(|length| length.checked_sub(format::TRAILER_LENGTH))
                .ok_or(GcRetirementIntentEncodeError::LengthOverflow)?,
        )
        .map_err(|source| GcRetirementIntentEncodeError::Allocation { source })?;
    for candidate in intent.candidates() {
        write_candidate(&mut candidates, candidate);
    }
    let candidate_set_digest = format::candidate_set_digest(candidate_count, &candidates);

    let mut encoded = Vec::new();
    encoded
        .try_reserve_exact(total_length)
        .map_err(|source| GcRetirementIntentEncodeError::Allocation { source })?;
    write_header(&mut encoded, intent, declared_length, candidate_set_digest);
    encoded.extend_from_slice(&candidates);
    let digest = format::intent_digest(&encoded);
    encoded.extend_from_slice(&digest);
    let checksum = format::checksum(&encoded);
    encoded.extend_from_slice(&checksum);
    Ok(CanonicalGcRetirementIntent::admitted(
        encoded,
        intent.clone(),
        GcCandidateSetDigest::from_verified(candidate_set_digest),
        GcRetirementIntentDigest::from_hash(digest),
    ))
}

/// The registered candidate-set digest over `candidates` in their given
/// order, without allocating the full record.
pub(super) fn candidate_set_digest(candidates: &[GcCandidate]) -> GcCandidateSetDigest {
    let mut bytes = Vec::new();
    for candidate in candidates {
        write_candidate(&mut bytes, candidate);
    }
    let count = u32::try_from(candidates.len()).unwrap_or(u32::MAX);
    GcCandidateSetDigest::from_verified(format::candidate_set_digest(count, &bytes))
}

fn write_candidate(output: &mut Vec<u8>, candidate: &GcCandidate) {
    output.extend_from_slice(candidate.segment_digest().as_bytes());
    output.extend_from_slice(&candidate.segment_length().to_be_bytes());
    output.extend_from_slice(candidate.evidence_digest().as_bytes());
}

fn write_header(
    output: &mut Vec<u8>,
    intent: &GcRetirementIntent,
    declared_length: u64,
    candidate_set_digest: [u8; 32],
) {
    let coordinates = intent.coordinates();
    output.extend_from_slice(&format::MAGIC);
    output.extend_from_slice(&format::VERSION.to_be_bytes());
    output.extend_from_slice(&format::RECORD_HEADER_LENGTH.to_be_bytes());
    output.extend_from_slice(&0_u32.to_be_bytes());
    output.extend_from_slice(&declared_length.to_be_bytes());
    output.extend_from_slice(&coordinates.generation.get().to_be_bytes());
    output.extend_from_slice(&format::RECORD_CANDIDATE_WIDTH.to_be_bytes());
    output.extend_from_slice(&0_u16.to_be_bytes());
    output.extend_from_slice(&intent.candidate_count().to_be_bytes());
    write_coordinates(output, coordinates);
    output.extend_from_slice(&candidate_set_digest);
}

fn write_coordinates(output: &mut Vec<u8>, coordinates: &GcRetirementIntentCoordinates) {
    output.extend_from_slice(&coordinates.liveness_generation.get().to_be_bytes());
    output.extend_from_slice(coordinates.manifest_digest.as_bytes());
    output.extend_from_slice(&coordinates.catalog_generation.get().to_be_bytes());
    output.extend_from_slice(coordinates.catalog_digest.as_bytes());
    output.extend_from_slice(&coordinates.profile.identity().to_be_bytes());
    output.extend_from_slice(&coordinates.profile.version().to_be_bytes());
    output.extend_from_slice(coordinates.profile.digest());
    output.extend_from_slice(coordinates.catalog_successor_proof_digest.as_bytes());
    output.extend_from_slice(coordinates.segment_pool_identity_digest.as_bytes());
    output.extend_from_slice(coordinates.disposition_set_digest.as_bytes());
    output.extend_from_slice(&coordinates.reader_lock.device().to_be_bytes());
    output.extend_from_slice(&coordinates.reader_lock.mount().to_be_bytes());
    output.extend_from_slice(&coordinates.reader_lock.file().to_be_bytes());
}
