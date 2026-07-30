//! Fingerprint-bound read-only recovery-stage assessment laws.

#[path = "recovery_stage_assessment/admission_laws.rs"]
mod admission_laws;
#[path = "recovery_stage_assessment/assessment_laws.rs"]
mod assessment_laws;
mod support;

use std::error::Error;

use keep::{
    LayoutEntryLimit, RecoveryStage, RecoveryStageEvidence, RecoveryStageMetadata,
    SegmentReadPolicy, SegmentRecordLimit, fingerprint_recovery_stage,
};
use support::decode_hex;

const SEGMENT_HEX: &str = include_str!("../conformance/segment-store/v1/one-zero-segment.hex");
const CATALOG_HEX: &str = include_str!("../conformance/segment-store/v1/one-zero-catalog.hex");
const HEAD_HEX: &str = include_str!("../conformance/segment-store/v1/one-zero-head.hex");

fn fixture(hex: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    decode_hex(
        hex.strip_suffix('\n')
            .ok_or("recovery fixture must end in one LF")?,
    )
    .map_err(Into::into)
}

fn evidence(stage: RecoveryStage, encoded: &[u8]) -> Result<RecoveryStageEvidence, Box<dyn Error>> {
    let length = u64::try_from(encoded.len())?;
    Ok(fingerprint_recovery_stage(
        RecoveryStageMetadata::new(stage, length)?,
        encoded,
    )?)
}

const fn maximum_policy() -> SegmentReadPolicy {
    SegmentReadPolicy::new(SegmentRecordLimit::MAXIMUM, LayoutEntryLimit::MAXIMUM)
}
