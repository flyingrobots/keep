//! Complete immutable-segment structural and content admission.

use super::segment_seal_envelope::SegmentSealEnvelope;
use super::{
    AdmittedSegment, SegmentHeader, SegmentReadError, SegmentReadPolicy, SegmentSeal,
    segment_identity_index,
};

const HEADER_LENGTH: usize = SegmentHeader::ENCODED_LENGTH;
const SEAL_LENGTH: usize = SegmentSeal::ENCODED_LENGTH;
const MINIMUM_LENGTH: usize = HEADER_LENGTH + SEAL_LENGTH;

pub(super) fn decode(
    encoded: &[u8],
    policy: SegmentReadPolicy,
) -> Result<AdmittedSegment<'_>, SegmentReadError> {
    if encoded.len() < MINIMUM_LENGTH {
        return Err(SegmentReadError::WrongLength {
            minimum: MINIMUM_LENGTH,
            observed: encoded.len(),
        });
    }
    let seal_offset =
        encoded
            .len()
            .checked_sub(SEAL_LENGTH)
            .ok_or(SegmentReadError::WrongLength {
                minimum: MINIMUM_LENGTH,
                observed: encoded.len(),
            })?;
    let header_bytes = encoded
        .get(..HEADER_LENGTH)
        .ok_or(SegmentReadError::WrongLength {
            minimum: MINIMUM_LENGTH,
            observed: encoded.len(),
        })?;
    let prefix = encoded
        .get(..seal_offset)
        .ok_or(SegmentReadError::WrongLength {
            minimum: MINIMUM_LENGTH,
            observed: encoded.len(),
        })?;
    let seal_bytes = encoded
        .get(seal_offset..)
        .ok_or(SegmentReadError::WrongLength {
            minimum: MINIMUM_LENGTH,
            observed: encoded.len(),
        })?;
    let records = encoded
        .get(HEADER_LENGTH..seal_offset)
        .ok_or(SegmentReadError::WrongLength {
            minimum: MINIMUM_LENGTH,
            observed: encoded.len(),
        })?;

    SegmentHeader::decode(header_bytes).map_err(|source| SegmentReadError::Header { source })?;
    let envelope = SegmentSealEnvelope::decode(prefix, seal_bytes)
        .map_err(|source| SegmentReadError::Seal { source })?;
    validate_record_count(envelope.record_count(), policy)?;
    segment_identity_index::validate(records, envelope.record_count(), policy)?;
    let seal = envelope
        .verify(prefix)
        .map_err(|source| SegmentReadError::Seal { source })?;
    Ok(AdmittedSegment::admitted(encoded, records, seal, policy))
}

const fn validate_record_count(
    record_count: u32,
    policy: SegmentReadPolicy,
) -> Result<(), SegmentReadError> {
    let maximum = policy.record_limit().get();
    let observed = record_count;
    if observed > maximum {
        return Err(SegmentReadError::RecordCountLimit { maximum, observed });
    }
    Ok(())
}
