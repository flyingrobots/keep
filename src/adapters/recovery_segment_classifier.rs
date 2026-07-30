//! This module owns whole-byte semantic classification of `current.seg`.

use super::{
    AdmittedSegment, RecoverySegmentStage, RecoverySegmentStageError, RecoverySegmentTruncation,
    RecoveryStage, RecoveryStageMetadata, ReusableRecoverySegment, SegmentHeader, SegmentReadError,
    SegmentReadPolicy, SegmentSeal, recovery_segment_fixed_framing, segment_identity_index,
    segment_record_cursor_decode, segment_record_header, segment_seal,
};

/// Classifies one complete caller-supplied segment-stage byte sequence.
///
/// The input must contain all currently observed `current.seg` bytes. The call
/// performs no I/O and does not retain or copy content bytes. Reusable and
/// complete states allocate a duplicate-identity index bounded by
/// `policy.record_limit()`.
///
/// # Errors
///
/// Returns [`RecoverySegmentStageError`] for oversized input, complete-looking
/// corruption, duplicate identities, resource refusal, arithmetic failure, or
/// an unsupported format coordinate. Known incomplete boundaries are returned
/// as [`RecoverySegmentStage::Truncated`] only while every available
/// fixed-framing byte remains canonical.
pub fn classify_recovery_segment_stage(
    encoded: &[u8],
    policy: SegmentReadPolicy,
) -> Result<RecoverySegmentStage<'_>, RecoverySegmentStageError> {
    let observed =
        u64::try_from(encoded.len()).map_err(|_| RecoverySegmentStageError::AddressSpace {
            observed: encoded.len(),
        })?;
    let metadata = RecoveryStageMetadata::new(RecoveryStage::Segment, observed)
        .map_err(|source| RecoverySegmentStageError::Metadata { source })?;
    let Some(header_bytes) = encoded.get(..SegmentHeader::ENCODED_LENGTH) else {
        recovery_segment_fixed_framing::segment_header(encoded)
            .map_err(|source| RecoverySegmentStageError::Header { source })?;
        return Ok(RecoverySegmentStage::Truncated(
            RecoverySegmentTruncation::Header {
                required: SegmentHeader::ENCODED_LENGTH,
                observed: encoded.len(),
            },
        ));
    };
    SegmentHeader::decode(header_bytes)
        .map_err(|source| RecoverySegmentStageError::Header { source })?;
    classify_tail(encoded, metadata.length(), policy)
}

fn classify_tail(
    encoded: &[u8],
    length: super::RecoveryStageLength,
    policy: SegmentReadPolicy,
) -> Result<RecoverySegmentStage<'_>, RecoverySegmentStageError> {
    let records = encoded.get(SegmentHeader::ENCODED_LENGTH..).ok_or(
        RecoverySegmentStageError::AddressSpace {
            observed: encoded.len(),
        },
    )?;
    let initial_offset = u64::try_from(SegmentHeader::ENCODED_LENGTH).map_err(|_| {
        RecoverySegmentStageError::AddressSpace {
            observed: SegmentHeader::ENCODED_LENGTH,
        }
    })?;
    let mut cursor = ReusableCursor::new(records, initial_offset);
    loop {
        if cursor.remaining.is_empty() {
            return admit_reusable(records, cursor.record_index, length, policy);
        }
        if is_seal_candidate(cursor.remaining) {
            if cursor.remaining.len() < SegmentSeal::ENCODED_LENGTH {
                return Ok(RecoverySegmentStage::Truncated(
                    RecoverySegmentTruncation::Seal {
                        offset: cursor.offset,
                        required: SegmentSeal::ENCODED_LENGTH,
                        observed: cursor.remaining.len(),
                    },
                ));
            }
            return AdmittedSegment::decode(encoded, policy)
                .map(RecoverySegmentStage::Complete)
                .map_err(|source| RecoverySegmentStageError::Complete { source });
        }
        if cursor.remaining.len() < segment_record_header::ENCODED_LENGTH {
            recovery_segment_fixed_framing::segment_tail(cursor.remaining).map_err(|source| {
                RecoverySegmentStageError::Record {
                    source: SegmentReadError::RecordHeader {
                        record_index: cursor.record_index,
                        offset: cursor.offset,
                        source,
                    },
                }
            })?;
        }
        match cursor.advance(policy) {
            Ok(()) => {}
            Err(source) => return classify_cursor_error(source),
        }
    }
}

fn admit_reusable<'a>(
    records: &[u8],
    record_count: u32,
    length: super::RecoveryStageLength,
    policy: SegmentReadPolicy,
) -> Result<RecoverySegmentStage<'a>, RecoverySegmentStageError> {
    segment_identity_index::validate(records, record_count, policy)
        .map_err(|source| RecoverySegmentStageError::Record { source })?;
    Ok(RecoverySegmentStage::Reusable(
        ReusableRecoverySegment::new(record_count, length),
    ))
}

const fn classify_cursor_error(
    source: SegmentReadError,
) -> Result<RecoverySegmentStage<'static>, RecoverySegmentStageError> {
    match source {
        SegmentReadError::RecordHeaderTruncated {
            record_index,
            offset,
            required,
            observed,
        } => Ok(RecoverySegmentStage::Truncated(
            RecoverySegmentTruncation::TailHeader {
                record_index,
                offset,
                required,
                observed,
            },
        )),
        SegmentReadError::RecordTruncated {
            record_index,
            offset,
            expected,
            observed,
        } => Ok(RecoverySegmentStage::Truncated(
            RecoverySegmentTruncation::Record {
                record_index,
                offset,
                expected,
                observed,
            },
        )),
        source => Err(RecoverySegmentStageError::Record { source }),
    }
}

fn is_seal_candidate(remaining: &[u8]) -> bool {
    has_magic(remaining, segment_seal::MAGIC)
        || (remaining.len() == SegmentSeal::ENCODED_LENGTH
            && !has_magic(remaining, segment_record_header::MAGIC))
}

fn has_magic(remaining: &[u8], expected: [u8; 16]) -> bool {
    let Some(magic) = remaining.first_chunk::<16>() else {
        return false;
    };
    *magic == expected
}

struct ReusableCursor<'a> {
    remaining: &'a [u8],
    record_index: u32,
    offset: u64,
}

impl<'a> ReusableCursor<'a> {
    const fn new(remaining: &'a [u8], offset: u64) -> Self {
        Self {
            remaining,
            record_index: 0,
            offset,
        }
    }

    fn advance(&mut self, policy: SegmentReadPolicy) -> Result<(), SegmentReadError> {
        let observed =
            self.record_index
                .checked_add(1)
                .ok_or(SegmentReadError::RecordIndexArithmetic {
                    record_index: self.record_index,
                })?;
        let maximum = policy.record_limit().get();
        if observed > maximum {
            return Err(SegmentReadError::RecordCountLimit { maximum, observed });
        }
        let decoded = segment_record_cursor_decode::decode(
            self.remaining,
            self.record_index,
            self.offset,
            policy,
        )?;
        self.remaining =
            self.remaining
                .get(decoded.host_length..)
                .ok_or(SegmentReadError::RecordTruncated {
                    record_index: self.record_index,
                    offset: self.offset,
                    expected: decoded.record_length,
                    observed: self.remaining.len(),
                })?;
        self.offset = self.offset.checked_add(decoded.record_length).ok_or(
            SegmentReadError::OffsetArithmetic {
                record_index: self.record_index,
                offset: self.offset,
                record_length: decoded.record_length,
            },
        )?;
        self.record_index = observed;
        Ok(())
    }
}
