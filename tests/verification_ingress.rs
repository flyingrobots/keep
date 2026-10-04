//! Raw segment verification admits bytes before issuing a typed report.
mod support;
use keep::{
    LayoutEntryLimit, SegmentHeaderError, SegmentReadError, SegmentReadPolicy, SegmentRecordLimit,
    SegmentSealError, VerificationDepth as Depth, VerificationError, VerificationRefusal,
    VerificationSource, verify_segment,
};
use std::error::Error;

const SEGMENT: &str = include_str!("../conformance/segment-store/v1/one-zero-segment.hex");

// Size: small. Oracle: supported physical depths over the frozen one-zero segment.
// Delete if raw segment verification is removed or stronger corpus laws subsume it.
#[test]
fn raw_segment_reports_are_only_issued_after_complete_admission() -> Result<(), Box<dyn Error>> {
    let bytes = support::decode_hex(SEGMENT.trim_end())?;
    for depth in [Depth::Framing, Depth::Checksum] {
        let report = verify_segment(&bytes, SegmentReadPolicy::MAXIMUM, depth)?;
        assert_eq!(report.requested(), depth);
        assert_eq!(
            report
                .subjects()
                .first()
                .ok_or("missing segment claim")?
                .depth(),
            depth
        );
    }
    Ok(())
}

// Size: small. Oracle: version one is required; a changed version is a typed
// protocol contradiction even when only framing evidence was requested.
// Delete if the segment format admission contract is deliberately replaced.
#[test]
fn segment_version_refusal_preserves_both_protocol_coordinates() -> Result<(), Box<dyn Error>> {
    let mut bytes = support::decode_hex(SEGMENT.trim_end())?;
    *bytes.get_mut(17).ok_or("version field absent")? = 2;
    let error = verify_segment(&bytes, SegmentReadPolicy::MAXIMUM, Depth::Framing)
        .err()
        .ok_or("unsupported version admitted")?;
    assert!(
        matches!(error, VerificationError::Refused { refusal: VerificationRefusal::Corrupt { .. }, source: Some(source) } if matches!(source.as_ref(), VerificationSource::Segment(SegmentReadError::Header { source: SegmentHeaderError::UnsupportedVersion { expected: 1, observed: 2 } }))),
        "exact original protocol contradiction required"
    );
    Ok(())
}

// Size: small. Oracle: exact stored versus canonical seal checksum coordinates.
// Delete when a stronger generated integrity law includes this witness.
#[test]
fn segment_checksum_refusal_preserves_expected_and_observed_bytes() -> Result<(), Box<dyn Error>> {
    let mut bytes = support::decode_hex(SEGMENT.trim_end())?;
    let offset = bytes.len().checked_sub(32).ok_or("seal checksum absent")?;
    let expected: [u8; 32] = bytes
        .get(offset..)
        .ok_or("seal checksum absent")?
        .try_into()?;
    *bytes.last_mut().ok_or("empty segment")? ^= 1;
    let observed: [u8; 32] = bytes
        .get(offset..)
        .ok_or("seal checksum absent")?
        .try_into()?;
    let error = verify_segment(&bytes, SegmentReadPolicy::MAXIMUM, Depth::Checksum)
        .err()
        .ok_or("corrupt seal admitted")?;
    assert!(
        matches!(error, VerificationError::Refused { refusal: VerificationRefusal::Corrupt { .. }, source: Some(source) } if matches!(source.as_ref(), VerificationSource::Segment(SegmentReadError::Seal { source: SegmentSealError::SealChecksumMismatch { expected: actual_expected, observed: actual_observed } }) if *actual_expected == expected && *actual_observed == observed)),
        "exact seal checksum source must survive"
    );
    Ok(())
}

// Size: small. Oracle: caller-selected admission capacity is not content corruption.
// Delete if segment resource policy is replaced with a stronger boundary contract.
#[test]
fn a_segment_policy_limit_remains_an_operational_failure() -> Result<(), Box<dyn Error>> {
    let bytes = support::decode_hex(SEGMENT.trim_end())?;
    let policy = SegmentReadPolicy::new(SegmentRecordLimit::new(0)?, LayoutEntryLimit::MAXIMUM);
    let error = verify_segment(&bytes, policy, Depth::Checksum)
        .err()
        .ok_or("record limit ignored")?;
    assert!(
        matches!(error, VerificationError::Operational { source } if matches!(source.as_ref(), VerificationSource::Segment(SegmentReadError::RecordCountLimit { maximum: 0, observed: 1 }))),
        "exact resource limit must survive without a corruption claim"
    );
    Ok(())
}
