//! Candidate-head truncation, admission, and refusal laws.

use std::error::Error;

use keep::{
    PublicationHeadDecodeError, RecoveryNextHeadStage, RecoveryNextHeadStageError, RecoveryStage,
    RecoveryStageMetadataError, classify_recovery_next_head_stage,
};

use super::{HEAD_HEX, HEAD_LENGTH, fixture};

#[test]
fn canonical_next_head_stage_is_complete() -> Result<(), Box<dyn Error>> {
    let encoded = fixture(HEAD_HEX)?;

    let RecoveryNextHeadStage::Complete(head) = classify_recovery_next_head_stage(&encoded)? else {
        return Err("canonical next-head stage was not complete".into());
    };

    assert_eq!(head.encoded(), encoded);
    assert_eq!(head.generation().get(), 1);
    Ok(())
}

#[test]
fn partial_next_head_is_exactly_truncated() -> Result<(), Box<dyn Error>> {
    let complete = fixture(HEAD_HEX)?;
    let observed = HEAD_LENGTH.checked_sub(1).ok_or("head underflow")?;
    let encoded = complete.get(..observed).ok_or("missing partial head")?;

    let state = classify_recovery_next_head_stage(encoded)?;

    assert!(matches!(
        state,
        RecoveryNextHeadStage::Truncated {
            required: HEAD_LENGTH,
            observed: actual,
        } if actual == observed
    ));
    Ok(())
}

#[test]
fn oversized_next_head_is_refused_before_decoding() -> Result<(), Box<dyn Error>> {
    let mut encoded = fixture(HEAD_HEX)?;
    encoded.push(0);

    let error = classify_recovery_next_head_stage(&encoded)
        .err()
        .ok_or("oversized next head was classified as lawful")?;

    assert!(matches!(
        error,
        RecoveryNextHeadStageError::Metadata {
            source: RecoveryStageMetadataError::Oversized {
                stage: RecoveryStage::NextHead,
                maximum: 128,
                observed: 129,
            },
        }
    ));
    Ok(())
}

#[test]
fn complete_invalid_next_head_is_a_typed_refusal() -> Result<(), Box<dyn Error>> {
    let mut encoded = fixture(HEAD_HEX)?;
    let byte = encoded.last_mut().ok_or("missing head checksum")?;
    *byte ^= 1;

    let error = classify_recovery_next_head_stage(&encoded)
        .err()
        .ok_or("invalid next head was classified as lawful")?;

    assert!(matches!(
        error,
        RecoveryNextHeadStageError::Complete {
            source: PublicationHeadDecodeError::ChecksumMismatch { .. },
        }
    ));
    Ok(())
}
