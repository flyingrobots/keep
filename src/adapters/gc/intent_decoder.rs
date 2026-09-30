//! This boundary module owns canonical GC retirement intent decoding order.

use super::GcRetirementIntentDecodeError as Error;
use super::{
    AdmittedGcRetirementIntent, GcRetirementIntent, GcRetirementIntentDigest,
    intent_candidate_decoder, intent_format as format, intent_header_decoder, intent_integrity,
    intent_semantic_header,
};

pub(super) fn decode(encoded: &[u8]) -> Result<AdmittedGcRetirementIntent<'_>, Error> {
    let header = intent_header_decoder::decode(encoded)?;
    let digest = intent_integrity::verify(encoded, header.digest_offset, header.checksum_offset)?;
    let candidate_bytes = encoded
        .get(format::HEADER_LENGTH..header.digest_offset)
        .ok_or(Error::Truncated {
            expected: header.digest_offset,
            observed: encoded.len(),
        })?;
    let candidate_set_digest = intent_integrity::verify_candidate_set(
        header.candidate_count,
        candidate_bytes,
        header.candidate_set_digest,
    )?;
    let coordinates = intent_semantic_header::admit(&header)?;
    let candidates = intent_candidate_decoder::decode(candidate_bytes, header.candidate_count)?;
    let intent = GcRetirementIntent::new(coordinates, candidates)
        .map_err(|source| Error::Semantic { source })?;
    Ok(AdmittedGcRetirementIntent::admitted(
        encoded,
        intent,
        candidate_set_digest,
        GcRetirementIntentDigest::from_hash(digest),
    ))
}
