//! Exact next-head finalization laws.

#[path = "recovery_next_head_finalization/execution_laws.rs"]
mod execution_laws;
#[path = "recovery_next_head_finalization/planning_laws.rs"]
mod planning_laws;
#[path = "recovery_next_head_finalization/storage_double.rs"]
pub mod storage_double;
mod support;

use std::error::Error;

use keep::{
    AdmittedSegment, CatalogPublicationExpectation, CatalogSnapshot, ChecksummedCatalog,
    ChecksummedPublicationHead, LayoutEntryLimit, RecoveryNextHeadFinalizationRequest,
    RecoveryStage, RecoveryStageAssessment, RecoveryStageEvidence, RecoveryStageMetadata,
    SegmentReadPolicy, SegmentRecordLimit, admit_recovery_stage_bytes, assess_recovery_stage,
    fingerprint_recovery_stage, plan_recovery_next_head_finalization,
};
use support::decode_hex;

const SEGMENT_HEX: &str = include_str!("../conformance/segment-store/v1/one-zero-segment.hex");
const CATALOG_ONE_HEX: &str = include_str!("../conformance/segment-store/v1/one-zero-catalog.hex");
const HEAD_ONE_HEX: &str = include_str!("../conformance/segment-store/v1/one-zero-head.hex");
const CATALOG_TWO_HEX: &str =
    include_str!("../conformance/segment-store/v1/one-zero-catalog-generation-two.hex");
const HEAD_TWO_HEX: &str =
    include_str!("../conformance/segment-store/v1/one-zero-head-generation-two.hex");

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

fn assessment(
    stage: RecoveryStage,
    encoded: &[u8],
) -> Result<RecoveryStageAssessment<'_>, Box<dyn Error>> {
    let observed = evidence(stage, encoded)?;
    let admitted = admit_recovery_stage_bytes(stage, observed, encoded)?;
    Ok(assess_recovery_stage(&admitted, maximum_policy())?)
}

fn snapshot<'bytes>(
    head_bytes: &'bytes [u8],
    catalog_bytes: &'bytes [u8],
    segment_bytes: &'bytes [u8],
) -> Result<CatalogSnapshot<'bytes, 'bytes, 'bytes>, Box<dyn Error>> {
    let segments = [AdmittedSegment::decode(segment_bytes, maximum_policy())?];
    let catalog = ChecksummedCatalog::decode(catalog_bytes)?.admit(&segments)?;
    Ok(ChecksummedPublicationHead::decode(head_bytes)?.admit(catalog)?)
}

fn initial_request(
    head_bytes: &[u8],
    catalog_bytes: &[u8],
    segment_bytes: &[u8],
) -> Result<RecoveryNextHeadFinalizationRequest, Box<dyn Error>> {
    let assessed = assessment(RecoveryStage::NextHead, head_bytes)?;
    let candidate = snapshot(head_bytes, catalog_bytes, segment_bytes)?;
    Ok(plan_recovery_next_head_finalization(
        &assessed,
        &candidate,
        CatalogPublicationExpectation::uninitialized(),
    )?)
}

const fn maximum_policy() -> SegmentReadPolicy {
    SegmentReadPolicy::new(SegmentRecordLimit::MAXIMUM, LayoutEntryLimit::MAXIMUM)
}
