//! Lawful reusable, complete, and truncated stage states.

use std::error::Error;

use keep::{RecoverySegmentStage, RecoverySegmentTruncation, classify_recovery_segment_stage};

use super::{
    EMPTY_SEGMENT_HEX, HEADER_LENGTH, ONE_ZERO_SEGMENT_HEX, RECORD_END, SEAL_LENGTH,
    maximum_policy, segment_bytes,
};

#[test]
fn canonical_header_is_a_reusable_empty_stage() -> Result<(), Box<dyn Error>> {
    let complete = segment_bytes(EMPTY_SEGMENT_HEX)?;
    let encoded = complete.get(..HEADER_LENGTH).ok_or("missing header")?;

    let RecoverySegmentStage::Reusable(stage) =
        classify_recovery_segment_stage(encoded, maximum_policy())?
    else {
        return Err("canonical empty prefix was not reusable".into());
    };

    assert_eq!(stage.record_count(), 0);
    assert_eq!(stage.length().get(), u64::try_from(HEADER_LENGTH)?);
    Ok(())
}

#[test]
fn canonical_record_prefix_is_reusable_without_rewriting() -> Result<(), Box<dyn Error>> {
    let complete = segment_bytes(ONE_ZERO_SEGMENT_HEX)?;
    let encoded = complete.get(..RECORD_END).ok_or("missing record prefix")?;

    let RecoverySegmentStage::Reusable(stage) =
        classify_recovery_segment_stage(encoded, maximum_policy())?
    else {
        return Err("canonical record prefix was not reusable".into());
    };

    assert_eq!(stage.record_count(), 1);
    assert_eq!(stage.length().get(), u64::try_from(RECORD_END)?);
    Ok(())
}

#[test]
fn canonical_sealed_stage_is_a_complete_segment() -> Result<(), Box<dyn Error>> {
    let encoded = segment_bytes(ONE_ZERO_SEGMENT_HEX)?;

    let RecoverySegmentStage::Complete(segment) =
        classify_recovery_segment_stage(&encoded, maximum_policy())?
    else {
        return Err("canonical sealed stage was not complete".into());
    };

    assert_eq!(segment.encoded(), encoded);
    assert_eq!(segment.record_count(), 1);
    Ok(())
}

#[test]
fn partial_fixed_header_is_exactly_truncated() -> Result<(), Box<dyn Error>> {
    let complete = segment_bytes(EMPTY_SEGMENT_HEX)?;
    let observed = HEADER_LENGTH.checked_sub(1).ok_or("header underflow")?;
    let encoded = complete.get(..observed).ok_or("missing partial header")?;

    let state = classify_recovery_segment_stage(encoded, maximum_policy())?;

    assert!(matches!(
        state,
        RecoverySegmentStage::Truncated(RecoverySegmentTruncation::Header {
            required: HEADER_LENGTH,
            observed: actual,
        }) if actual == observed
    ));
    Ok(())
}

#[test]
fn partial_record_header_is_exactly_truncated() -> Result<(), Box<dyn Error>> {
    let complete = segment_bytes(ONE_ZERO_SEGMENT_HEX)?;
    let tail_length = 100_usize;
    let observed_end = HEADER_LENGTH
        .checked_add(tail_length)
        .ok_or("record-header prefix overflow")?;
    let encoded = complete
        .get(..observed_end)
        .ok_or("missing partial record header")?;

    let state = classify_recovery_segment_stage(encoded, maximum_policy())?;

    assert!(matches!(
        state,
        RecoverySegmentStage::Truncated(RecoverySegmentTruncation::TailHeader {
            record_index: 0,
            offset: 64,
            required: 112,
            observed: actual,
        }) if actual == tail_length
    ));
    Ok(())
}

#[test]
fn partial_record_body_is_exactly_truncated() -> Result<(), Box<dyn Error>> {
    let complete = segment_bytes(ONE_ZERO_SEGMENT_HEX)?;
    let observed_end = RECORD_END.checked_sub(1).ok_or("record underflow")?;
    let encoded = complete
        .get(..observed_end)
        .ok_or("missing partial record")?;

    let state = classify_recovery_segment_stage(encoded, maximum_policy())?;

    assert!(matches!(
        state,
        RecoverySegmentStage::Truncated(RecoverySegmentTruncation::Record {
            record_index: 0,
            offset: 64,
            expected: 145,
            observed: 144,
        })
    ));
    Ok(())
}

#[test]
fn partial_seal_is_exactly_truncated() -> Result<(), Box<dyn Error>> {
    let complete = segment_bytes(ONE_ZERO_SEGMENT_HEX)?;
    let partial_seal = SEAL_LENGTH.checked_div(2).ok_or("seal divisor")?;
    let observed_end = RECORD_END
        .checked_add(partial_seal)
        .ok_or("seal prefix overflow")?;
    let encoded = complete.get(..observed_end).ok_or("missing partial seal")?;

    let state = classify_recovery_segment_stage(encoded, maximum_policy())?;

    assert!(matches!(
        state,
        RecoverySegmentStage::Truncated(RecoverySegmentTruncation::Seal {
            offset: 209,
            required: SEAL_LENGTH,
            observed: actual,
        }) if actual == partial_seal
    ));
    Ok(())
}
