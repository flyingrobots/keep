//! Two-phase seal admission for corruption-localizing segment reads.

use super::segment_seal_decoder::DecodedSeal;
use super::{
    SegmentDigest, SegmentSeal, SegmentSealError, segment_seal_admission, segment_seal_hash,
};

pub(super) fn admit(
    prefix: &[u8],
    encoded: &[u8],
    fields: &DecodedSeal,
) -> Result<(), SegmentSealError> {
    segment_seal_admission::validate_fields(prefix, fields)?;
    let expected_checksum = segment_seal_hash::seal_checksum(encoded)?;
    if fields.checksum != expected_checksum {
        return Err(SegmentSealError::SealChecksumMismatch {
            expected: expected_checksum,
            observed: fields.checksum,
        });
    }
    Ok(())
}

pub(super) fn verify(prefix: &[u8], fields: &DecodedSeal) -> Result<SegmentSeal, SegmentSealError> {
    let canonical = segment_seal_admission::from_prefix(prefix, fields.record_count)?;
    let observed_digest = SegmentDigest::from_validated(fields.digest);
    if observed_digest != canonical.digest() {
        return Err(SegmentSealError::SegmentDigestMismatch {
            expected: canonical.digest(),
            observed: observed_digest,
        });
    }
    if fields.checksum != canonical.checksum() {
        return Err(SegmentSealError::SealChecksumMismatch {
            expected: canonical.checksum(),
            observed: fields.checksum,
        });
    }
    Ok(canonical)
}
