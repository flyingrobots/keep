//! This boundary module owns GC retirement intent digest and checksum
//! verification.

use super::GcRetirementIntentDecodeError as Error;
use super::intent_field_decoder::read_array;
use super::{GcCandidateSetDigest, intent_format as format};

/// Verifies the trailing checksum, then the intent digest, and returns the
/// verified intent digest bytes.
pub(super) fn verify(
    encoded: &[u8],
    digest_offset: usize,
    checksum_offset: usize,
) -> Result<[u8; 32], Error> {
    let observed_checksum: [u8; 32] = read_array(encoded, checksum_offset)?;
    let checksum_preimage = encoded.get(..checksum_offset).ok_or(Error::Truncated {
        expected: checksum_offset,
        observed: encoded.len(),
    })?;
    let expected_checksum = format::checksum(checksum_preimage);
    if observed_checksum != expected_checksum {
        return Err(Error::ChecksumMismatch {
            expected: expected_checksum,
            observed: observed_checksum,
        });
    }

    let observed_digest: [u8; 32] = read_array(encoded, digest_offset)?;
    let digest_preimage = encoded.get(..digest_offset).ok_or(Error::Truncated {
        expected: digest_offset,
        observed: encoded.len(),
    })?;
    let expected_digest = format::intent_digest(digest_preimage);
    if observed_digest != expected_digest {
        return Err(Error::IntentDigestMismatch {
            expected: expected_digest,
            observed: observed_digest,
        });
    }
    Ok(expected_digest)
}

pub(super) fn verify_candidate_set(
    candidate_count: u32,
    candidates: &[u8],
    observed: [u8; 32],
) -> Result<GcCandidateSetDigest, Error> {
    let expected = format::candidate_set_digest(candidate_count, candidates);
    if observed == expected {
        Ok(GcCandidateSetDigest::from_verified(expected))
    } else {
        Err(Error::CandidateSetDigestMismatch { expected, observed })
    }
}
