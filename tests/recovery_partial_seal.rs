//! Runtime refusal laws for contradictory incomplete segment seals.

mod support;

use std::error::Error;

use keep::{
    RecoveryStage, RecoveryStageMetadata, SegmentReadPolicy, SegmentSealError,
    admit_recovery_stage_bytes, assess_recovery_stage, classify_recovery_segment_stage,
    fingerprint_recovery_stage,
};

// Size: small. Oracle: KEEP-RECOVERY-010 and the v1 seal version is exactly 1.
// Delete only if a stronger public corruption law subsumes this counterexample.
#[test]
fn unsupported_partial_seal_version_refuses_classification() -> Result<(), Box<dyn Error>> {
    let bytes = unsupported_version_prefix()?;
    let error = classify_recovery_segment_stage(&bytes, SegmentReadPolicy::MAXIMUM)
        .err()
        .ok_or("unsupported seal version admitted as discardable truncation")?;
    assert_eq!(
        seal_cause(&error),
        Some(&SegmentSealError::UnsupportedVersion {
            expected: 1,
            observed: 2
        })
    );
    Ok(())
}

// Size: small. Oracle: fingerprint-bound corruption cannot authorize discard.
// Delete only if another assessment law preserves this precise refusal and input.
#[test]
fn unsupported_partial_seal_version_refuses_discard_assessment() -> Result<(), Box<dyn Error>> {
    let bytes = unsupported_version_prefix()?;
    let original = bytes.clone();
    let stage = RecoveryStage::Segment;
    let metadata = RecoveryStageMetadata::new(stage, u64::try_from(bytes.len())?)?;
    let fingerprint = fingerprint_recovery_stage(metadata, bytes.as_slice())?;
    let admitted = admit_recovery_stage_bytes(stage, fingerprint, &bytes)?;
    let error = assess_recovery_stage(&admitted, SegmentReadPolicy::MAXIMUM)
        .err()
        .ok_or("contradictory seal received a discardable assessment")?;
    assert_eq!(
        seal_cause(&error),
        Some(&SegmentSealError::UnsupportedVersion {
            expected: 1,
            observed: 2
        })
    );
    assert_eq!(
        bytes, original,
        "assessment must preserve the supplied evidence"
    );
    Ok(())
}

fn unsupported_version_prefix() -> Result<Vec<u8>, Box<dyn Error>> {
    let mut bytes = support::decode_hex(
        include_str!("../conformance/segment-store/v1/one-zero-segment.hex").trim(),
    )?;
    let version = bytes
        .len()
        .checked_sub(128)
        .and_then(|start| start.checked_add(17))
        .ok_or("fixture has no seal version")?;
    *bytes
        .get_mut(version)
        .ok_or("fixture has no version byte")? = 2;
    bytes.truncate(version.checked_add(1).ok_or("prefix length overflow")?);
    Ok(bytes)
}

fn seal_cause<'a>(error: &'a (dyn Error + 'static)) -> Option<&'a SegmentSealError> {
    let mut current = Some(error);
    while let Some(cause) = current {
        if let Some(seal) = cause.downcast_ref::<SegmentSealError>() {
            return Some(seal);
        }
        current = cause.source();
    }
    None
}
