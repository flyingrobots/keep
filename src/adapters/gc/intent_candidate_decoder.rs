//! This boundary module owns canonical GC candidate body decoding.

use super::GcRetirementIntentDecodeError as Error;
use super::intent_field_decoder::{read_array, read_u64, require_exact};
use super::{GcCandidate, VerificationEvidenceDigest, intent_format as format};
use crate::adapters::SegmentDigest;

pub(super) fn decode(encoded: &[u8], candidate_count: u32) -> Result<Vec<GcCandidate>, Error> {
    let capacity = usize::try_from(candidate_count).map_err(|_| Error::LengthOverflow)?;
    let expected_length = capacity
        .checked_mul(format::CANDIDATE_WIDTH)
        .ok_or(Error::LengthOverflow)?;
    require_exact(encoded, expected_length)?;
    let mut candidates = Vec::new();
    candidates
        .try_reserve_exact(capacity)
        .map_err(|source| Error::Allocation { source })?;
    for bytes in encoded.chunks_exact(format::CANDIDATE_WIDTH) {
        let segment_digest = SegmentDigest::from_validated(read_array(bytes, 0)?);
        let segment_length = read_u64(bytes, 32)?;
        let evidence_digest = VerificationEvidenceDigest::new(read_array(bytes, 40)?);
        candidates.push(GcCandidate::new(
            segment_digest,
            segment_length,
            evidence_digest,
        ));
    }
    Ok(candidates)
}
