//! Exact stage, length, and fingerprint binding laws.

use std::error::Error;

use keep::{RecoveryStage, RecoveryStageByteAdmissionError, admit_recovery_stage_bytes};

use super::{SEGMENT_HEX, evidence, fixture};

#[test]
fn exact_stage_bytes_retain_their_prior_evidence() -> Result<(), Box<dyn Error>> {
    let encoded = fixture(SEGMENT_HEX)?;
    let expected = evidence(RecoveryStage::Segment, &encoded)?;

    let admitted = admit_recovery_stage_bytes(RecoveryStage::Segment, expected, &encoded)?;

    assert_eq!(admitted.stage(), RecoveryStage::Segment);
    assert_eq!(admitted.evidence(), expected);
    assert_eq!(admitted.encoded(), encoded);
    Ok(())
}

#[test]
fn canonical_name_stage_mismatch_refuses_before_byte_admission() -> Result<(), Box<dyn Error>> {
    let encoded = fixture(SEGMENT_HEX)?;
    let observed = evidence(RecoveryStage::Segment, &encoded)?;

    let error = admit_recovery_stage_bytes(RecoveryStage::Catalog, observed, &encoded)
        .err()
        .ok_or("wrong canonical stage was admitted")?;

    assert!(matches!(
        error,
        RecoveryStageByteAdmissionError::StageMismatch {
            expected: RecoveryStage::Catalog,
            observed: RecoveryStage::Segment,
        }
    ));
    Ok(())
}

#[test]
fn changed_length_refuses_before_fingerprint_comparison() -> Result<(), Box<dyn Error>> {
    let encoded = fixture(SEGMENT_HEX)?;
    let expected = evidence(RecoveryStage::Segment, &encoded)?;
    let changed_length = encoded.len().checked_sub(1).ok_or("segment underflow")?;
    let changed = encoded
        .get(..changed_length)
        .ok_or("missing truncated segment")?;

    let error = admit_recovery_stage_bytes(RecoveryStage::Segment, expected, changed)
        .err()
        .ok_or("changed stage length was admitted")?;

    assert!(matches!(
        error,
        RecoveryStageByteAdmissionError::LengthMismatch {
            stage: RecoveryStage::Segment,
            expected: expected_length,
            observed,
        } if expected_length.get() == u64::try_from(encoded.len())?
            && observed == u64::try_from(changed.len())?
    ));
    Ok(())
}

#[test]
fn same_length_mutation_refuses_by_fingerprint() -> Result<(), Box<dyn Error>> {
    let mut encoded = fixture(SEGMENT_HEX)?;
    let expected = evidence(RecoveryStage::Segment, &encoded)?;
    let byte = encoded.last_mut().ok_or("missing segment byte")?;
    *byte ^= 1;

    let error = admit_recovery_stage_bytes(RecoveryStage::Segment, expected, &encoded)
        .err()
        .ok_or("changed stage fingerprint was admitted")?;

    assert!(matches!(
        error,
        RecoveryStageByteAdmissionError::FingerprintMismatch {
            stage: RecoveryStage::Segment,
            expected: expected_fingerprint,
            observed,
        } if expected_fingerprint == expected.fingerprint() && observed != expected_fingerprint
    ));
    Ok(())
}
