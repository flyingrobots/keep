//! Exact reusable-segment recovery laws.

#[path = "recovery_segment_resume/execution_laws.rs"]
mod execution_laws;
#[path = "recovery_segment_resume/planning_laws.rs"]
mod planning_laws;
#[path = "recovery_segment_resume/storage_double.rs"]
pub mod storage_double;
#[path = "support/mod.rs"]
mod support;

use std::error::Error;

use keep::{
    LayoutEntryLimit, RecoverySegmentResumeRequest, RecoveryStage, RecoveryStageAssessment,
    RecoveryStageEvidence, RecoveryStageMetadata, SegmentReadPolicy, SegmentRecordLimit,
    admit_recovery_stage_bytes, assess_recovery_stage, fingerprint_recovery_stage,
    plan_recovery_segment_resume,
};
use support::decode_hex;

const SEGMENT_HEX: &str = include_str!("../conformance/segment-store/v1/one-zero-segment.hex");
const HEAD_HEX: &str = include_str!("../conformance/segment-store/v1/one-zero-head.hex");
const SEGMENT_HEADER_LENGTH: usize = 64;
const SEGMENT_SEAL_LENGTH: usize = 128;

fn fixture(hex: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    decode_hex(
        hex.strip_suffix('\n')
            .ok_or("recovery fixture must end in one LF")?,
    )
    .map_err(Into::into)
}

fn reusable_prefix() -> Result<Vec<u8>, Box<dyn Error>> {
    let mut encoded = fixture(SEGMENT_HEX)?;
    let prefix_length = encoded
        .len()
        .checked_sub(SEGMENT_SEAL_LENGTH)
        .ok_or("segment fixture is shorter than its seal")?;
    encoded.truncate(prefix_length);
    Ok(encoded)
}

fn evidence(stage: RecoveryStage, encoded: &[u8]) -> Result<RecoveryStageEvidence, Box<dyn Error>> {
    let length = u64::try_from(encoded.len())?;
    Ok(fingerprint_recovery_stage(
        RecoveryStageMetadata::new(stage, length)?,
        encoded,
    )?)
}

fn assessment(
    stage: RecoveryStage,
    encoded: &[u8],
) -> Result<RecoveryStageAssessment<'_>, Box<dyn Error>> {
    let observed = evidence(stage, encoded)?;
    let admitted = admit_recovery_stage_bytes(stage, observed, encoded)?;
    Ok(assess_recovery_stage(&admitted, maximum_policy())?)
}

fn resume_request(encoded: &[u8]) -> Result<RecoverySegmentResumeRequest, Box<dyn Error>> {
    Ok(plan_recovery_segment_resume(
        &assessment(RecoveryStage::Segment, encoded)?,
        maximum_policy(),
    )?)
}

const fn maximum_policy() -> SegmentReadPolicy {
    SegmentReadPolicy::new(SegmentRecordLimit::MAXIMUM, LayoutEntryLimit::MAXIMUM)
}
