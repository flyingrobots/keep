//! Explicit fingerprint-bound stage-discard laws.

#[path = "recovery_stage_discard/execution_laws.rs"]
mod execution_laws;
#[path = "recovery_stage_discard/planning_laws.rs"]
mod planning_laws;
#[path = "recovery_stage_discard/storage_double.rs"]
pub mod storage_double;
mod support;

use std::error::Error;

use keep::{
    LayoutEntryLimit, RecoveryStage, RecoveryStageAssessment, RecoveryStageDiscardRequest,
    RecoveryStageEvidence, RecoveryStageMetadata, SegmentReadPolicy, SegmentRecordLimit,
    admit_recovery_stage_bytes, assess_recovery_stage, fingerprint_recovery_stage,
    plan_recovery_stage_discard,
};
use support::decode_hex;

const SEGMENT_HEX: &str = include_str!("../conformance/segment-store/v1/one-zero-segment.hex");
const CATALOG_HEX: &str = include_str!("../conformance/segment-store/v1/one-zero-catalog.hex");
const HEAD_HEX: &str = include_str!("../conformance/segment-store/v1/one-zero-head.hex");
const SEGMENT_SEAL_LENGTH: usize = 128;

fn fixture(hex: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    decode_hex(
        hex.strip_suffix('\n')
            .ok_or("recovery fixture must end in one LF")?,
    )
    .map_err(Into::into)
}

fn truncated_fixture(stage: RecoveryStage) -> Result<Vec<u8>, Box<dyn Error>> {
    let complete = fixture(match stage {
        RecoveryStage::Segment => SEGMENT_HEX,
        RecoveryStage::Catalog => CATALOG_HEX,
        RecoveryStage::NextHead => HEAD_HEX,
    })?;
    complete
        .get(..1)
        .map(<[u8]>::to_vec)
        .ok_or_else(|| "canonical recovery fixture is unexpectedly empty".into())
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

fn discard_request(
    stage: RecoveryStage,
    encoded: &[u8],
) -> Result<RecoveryStageDiscardRequest, Box<dyn Error>> {
    Ok(plan_recovery_stage_discard(&assessment(stage, encoded)?)?)
}

const fn maximum_policy() -> SegmentReadPolicy {
    SegmentReadPolicy::new(SegmentRecordLimit::MAXIMUM, LayoutEntryLimit::MAXIMUM)
}
