//! Complete-segment outer framing, count, and resource laws.

use std::error::Error;

use keep::{
    AdmittedSegment, LayoutEntryLimit, SegmentReadError, SegmentReadPolicy, SegmentRecordLimit,
    SegmentSealError,
};

use super::format_oracle::seal_segment;
use super::{
    EMPTY_SEGMENT_HEX, ONE_ZERO_SEGMENT_HEX, maximum_policy, one_record_prefix, segment_bytes,
};

#[test]
fn independent_seal_oracle_reconstructs_the_frozen_segment() -> Result<(), Box<dyn Error>> {
    let canonical = segment_bytes(ONE_ZERO_SEGMENT_HEX)?;
    let prefix = one_record_prefix()?;
    assert_eq!(seal_segment(&prefix, 1)?, canonical);
    Ok(())
}

#[test]
fn configured_record_limit_refuses_before_identity_index_allocation() -> Result<(), Box<dyn Error>>
{
    let encoded = segment_bytes(ONE_ZERO_SEGMENT_HEX)?;
    let zero = SegmentRecordLimit::new(0)?;
    let policy = SegmentReadPolicy::new(zero, LayoutEntryLimit::MAXIMUM);
    let error = refusal(&encoded, policy)?;

    let SegmentReadError::RecordCountLimit { maximum, observed } = error else {
        return Err(format!("unexpected record-limit refusal: {error}").into());
    };
    assert_eq!(maximum, 0);
    assert_eq!(observed, 1);
    Ok(())
}

#[test]
fn configured_record_limit_cannot_exceed_the_format_bound() -> Result<(), Box<dyn Error>> {
    let error = match SegmentRecordLimit::new(1_048_577) {
        Ok(_limit) => return Err("out-of-bounds segment record limit was admitted".into()),
        Err(error) => error,
    };

    assert_eq!(error.maximum(), 1_048_576);
    assert_eq!(error.observed(), 1_048_577);
    Ok(())
}

#[test]
fn complete_segment_refuses_a_partial_fixed_envelope() -> Result<(), Box<dyn Error>> {
    let canonical = segment_bytes(EMPTY_SEGMENT_HEX)?;
    let truncated = canonical
        .get(..191)
        .ok_or("empty segment fixture lacks its truncation target")?;
    let error = refusal(truncated, maximum_policy())?;

    let SegmentReadError::WrongLength { minimum, observed } = error else {
        return Err(format!("unexpected envelope refusal: {error}").into());
    };
    assert_eq!(minimum, 192);
    assert_eq!(observed, 191);
    Ok(())
}

#[test]
fn coherent_seal_still_refuses_a_partial_record_tail() -> Result<(), Box<dyn Error>> {
    let prefix = one_record_prefix()?;
    let truncated = prefix
        .get(..176)
        .ok_or("segment fixture lacks its partial-record target")?;
    let encoded = seal_segment(truncated, 1)?;
    let error = refusal(&encoded, maximum_policy())?;

    let SegmentReadError::RecordTruncated {
        record_index,
        offset,
        expected,
        observed,
    } = error
    else {
        return Err(format!("unexpected partial-record refusal: {error}").into());
    };
    assert_eq!((record_index, offset), (0, 64));
    assert_eq!(expected, 145);
    assert_eq!(observed, 112);
    Ok(())
}

#[test]
fn bytes_appended_after_the_terminal_seal_are_never_ignored() -> Result<(), Box<dyn Error>> {
    let mut encoded = segment_bytes(EMPTY_SEGMENT_HEX)?;
    encoded.push(0);
    let error = refusal(&encoded, maximum_policy())?;

    let SegmentReadError::Seal {
        source: SegmentSealError::InvalidMagic { .. },
    } = error
    else {
        return Err(format!("unexpected post-seal refusal: {error}").into());
    };
    Ok(())
}

#[test]
fn declared_count_below_physical_records_refuses_the_exact_tail() -> Result<(), Box<dyn Error>> {
    let prefix = one_record_prefix()?;
    let encoded = seal_segment(&prefix, 0)?;
    let error = refusal(&encoded, maximum_policy())?;

    let SegmentReadError::TrailingRecordBytes { offset, observed } = error else {
        return Err(format!("unexpected low-count refusal: {error}").into());
    };
    assert_eq!(offset, 64);
    assert_eq!(observed, 145);
    Ok(())
}

#[test]
fn declared_count_above_physical_records_refuses_the_missing_header() -> Result<(), Box<dyn Error>>
{
    let prefix = one_record_prefix()?;
    let encoded = seal_segment(&prefix, 2)?;
    let error = refusal(&encoded, maximum_policy())?;

    let SegmentReadError::RecordHeaderTruncated {
        record_index,
        offset,
        required,
        observed,
    } = error
    else {
        return Err(format!("unexpected high-count refusal: {error}").into());
    };
    assert_eq!(record_index, 1);
    assert_eq!(offset, 209);
    assert_eq!(required, 112);
    assert_eq!(observed, 0);
    Ok(())
}

fn refusal(encoded: &[u8], policy: SegmentReadPolicy) -> Result<SegmentReadError, Box<dyn Error>> {
    match AdmittedSegment::decode(encoded, policy) {
        Ok(_admitted) => Err("malformed complete segment was admitted".into()),
        Err(error) => Ok(error),
    }
}
