//! Complete-looking corruption and bounded-resource refusal laws.

use std::error::Error;

use keep::{
    LayoutEntryLimit, RecoverySegmentStageError, SegmentHeaderError, SegmentReadError,
    SegmentReadPolicy, SegmentRecordHeaderError, SegmentRecordLimit,
    classify_recovery_segment_stage,
};

use super::{HEADER_LENGTH, ONE_ZERO_SEGMENT_HEX, RECORD_END, maximum_policy, segment_bytes};

#[test]
fn complete_invalid_header_is_a_typed_refusal() -> Result<(), Box<dyn Error>> {
    let mut encoded = segment_bytes(ONE_ZERO_SEGMENT_HEX)?;
    let byte = encoded.first_mut().ok_or("missing header byte")?;
    *byte ^= 1;

    let error = classify_recovery_segment_stage(&encoded, maximum_policy())
        .err()
        .ok_or("corrupt header was classified as lawful")?;

    assert!(matches!(error, RecoverySegmentStageError::Header { .. }));
    Ok(())
}

#[test]
fn every_corrupt_available_segment_header_byte_is_refused() -> Result<(), Box<dyn Error>> {
    let complete = segment_bytes(ONE_ZERO_SEGMENT_HEX)?;
    for offset in 0..HEADER_LENGTH {
        let end = offset.checked_add(1).ok_or("segment-header end overflow")?;
        let mut encoded = complete
            .get(..end)
            .ok_or("missing segment-header prefix")?
            .to_vec();
        let byte = encoded
            .get_mut(offset)
            .ok_or("missing segment-header byte")?;
        *byte ^= 1;

        let error = classify_recovery_segment_stage(&encoded, maximum_policy())
            .err()
            .ok_or("corrupt partial segment header was classified as truncation")?;

        assert!(matches!(
            error,
            RecoverySegmentStageError::Header {
                source: SegmentHeaderError::InvalidMagic { .. }
                    | SegmentHeaderError::UnsupportedVersion { .. }
                    | SegmentHeaderError::UnknownFlags { .. }
                    | SegmentHeaderError::HeaderLength { .. }
                    | SegmentHeaderError::RecordHeaderLength { .. }
                    | SegmentHeaderError::SealLength { .. }
                    | SegmentHeaderError::ReservedU16 { .. }
                    | SegmentHeaderError::MaximumRecordPayloadLength { .. }
                    | SegmentHeaderError::MaximumSegmentLength { .. }
                    | SegmentHeaderError::MaximumRecordCount { .. }
                    | SegmentHeaderError::RecordChecksumAlgorithm { .. }
                    | SegmentHeaderError::SegmentDigestAlgorithm { .. }
                    | SegmentHeaderError::ReservedBytes { .. },
            }
        ));
    }
    Ok(())
}

#[test]
fn every_corrupt_available_record_framing_byte_is_refused() -> Result<(), Box<dyn Error>> {
    let complete = segment_bytes(ONE_ZERO_SEGMENT_HEX)?;
    for offset in (0..24).chain(40..48).chain(108..112) {
        let end = HEADER_LENGTH
            .checked_add(offset)
            .and_then(|value| value.checked_add(1))
            .ok_or("record-header end overflow")?;
        let mut encoded = complete
            .get(..end)
            .ok_or("missing record-header prefix")?
            .to_vec();
        let absolute = HEADER_LENGTH
            .checked_add(offset)
            .ok_or("record-header offset overflow")?;
        let byte = encoded
            .get_mut(absolute)
            .ok_or("missing record-header byte")?;
        *byte ^= 1;

        let error = classify_recovery_segment_stage(&encoded, maximum_policy())
            .err()
            .ok_or("corrupt partial record header was classified as truncation")?;

        assert!(matches!(
            error,
            RecoverySegmentStageError::Record {
                source: SegmentReadError::RecordHeader {
                    record_index: 0,
                    offset: 64,
                    source: SegmentRecordHeaderError::InvalidMagic { .. }
                        | SegmentRecordHeaderError::UnsupportedVersion { .. }
                        | SegmentRecordHeaderError::UnknownRecordKind { .. }
                        | SegmentRecordHeaderError::UnknownFlags { .. }
                        | SegmentRecordHeaderError::HeaderLength { .. }
                        | SegmentRecordHeaderError::IdentityLength { .. }
                        | SegmentRecordHeaderError::RecordChecksumAlgorithm { .. }
                        | SegmentRecordHeaderError::IdentityVersion { .. }
                        | SegmentRecordHeaderError::IdentityAlgorithm { .. }
                        | SegmentRecordHeaderError::ReservedBytes { .. },
                },
            }
        ));
    }
    Ok(())
}

#[test]
fn complete_invalid_record_is_a_typed_refusal() -> Result<(), Box<dyn Error>> {
    let mut encoded = segment_bytes(ONE_ZERO_SEGMENT_HEX)?;
    let checksum_byte = encoded
        .get_mut(RECORD_END.checked_sub(1).ok_or("checksum underflow")?)
        .ok_or("missing record checksum")?;
    *checksum_byte ^= 1;

    let error = classify_recovery_segment_stage(&encoded, maximum_policy())
        .err()
        .ok_or("corrupt record was classified as lawful")?;

    assert!(matches!(
        error,
        RecoverySegmentStageError::Record {
            source: SegmentReadError::RecordDecode {
                record_index: 0,
                offset: 64,
                ..
            },
        }
    ));
    Ok(())
}

#[test]
fn complete_invalid_seal_is_a_typed_refusal() -> Result<(), Box<dyn Error>> {
    let mut encoded = segment_bytes(ONE_ZERO_SEGMENT_HEX)?;
    let seal_byte = encoded.last_mut().ok_or("missing seal checksum")?;
    *seal_byte ^= 1;

    let error = classify_recovery_segment_stage(&encoded, maximum_policy())
        .err()
        .ok_or("corrupt seal was classified as lawful")?;

    assert!(matches!(
        error,
        RecoverySegmentStageError::Complete {
            source: SegmentReadError::Seal { .. },
        }
    ));
    Ok(())
}

#[test]
fn fixed_width_tail_with_invalid_seal_magic_stays_a_seal_refusal() -> Result<(), Box<dyn Error>> {
    let mut encoded = segment_bytes(ONE_ZERO_SEGMENT_HEX)?;
    let magic_byte = encoded.get_mut(RECORD_END).ok_or("missing seal magic")?;
    *magic_byte ^= 1;

    let error = classify_recovery_segment_stage(&encoded, maximum_policy())
        .err()
        .ok_or("invalid seal magic was classified as lawful")?;

    assert!(matches!(
        error,
        RecoverySegmentStageError::Complete {
            source: SegmentReadError::Seal { .. },
        }
    ));
    Ok(())
}

#[test]
fn duplicate_reusable_records_are_never_resumable() -> Result<(), Box<dyn Error>> {
    let complete = segment_bytes(ONE_ZERO_SEGMENT_HEX)?;
    let header = complete.get(..HEADER_LENGTH).ok_or("missing header")?;
    let record = complete
        .get(HEADER_LENGTH..RECORD_END)
        .ok_or("missing record")?;
    let mut encoded = Vec::with_capacity(
        HEADER_LENGTH
            .checked_add(record.len().checked_mul(2).ok_or("record overflow")?)
            .ok_or("stage overflow")?,
    );
    encoded.extend_from_slice(header);
    encoded.extend_from_slice(record);
    encoded.extend_from_slice(record);

    let error = classify_recovery_segment_stage(&encoded, maximum_policy())
        .err()
        .ok_or("duplicate record stage was reusable")?;

    assert!(matches!(
        error,
        RecoverySegmentStageError::Record {
            source: SegmentReadError::DuplicateRecordIdentity { .. },
        }
    ));
    Ok(())
}

#[test]
fn reusable_record_count_obeys_the_caller_limit() -> Result<(), Box<dyn Error>> {
    let complete = segment_bytes(ONE_ZERO_SEGMENT_HEX)?;
    let encoded = complete.get(..RECORD_END).ok_or("missing record prefix")?;
    let policy = SegmentReadPolicy::new(SegmentRecordLimit::new(0)?, LayoutEntryLimit::MAXIMUM);

    let error = classify_recovery_segment_stage(encoded, policy)
        .err()
        .ok_or("record above caller limit was reusable")?;

    assert!(matches!(
        error,
        RecoverySegmentStageError::Record {
            source: SegmentReadError::RecordCountLimit {
                maximum: 0,
                observed: 1,
            },
        }
    ));
    Ok(())
}
