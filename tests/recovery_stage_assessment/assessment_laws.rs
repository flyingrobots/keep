//! Name-selected semantic dispatch laws.

use std::error::Error;

use keep::{
    CatalogDecodeError, RecoveryCatalogStage, RecoveryNextHeadStage, RecoverySegmentStage,
    RecoveryStage, RecoveryStageAssessment, RecoveryStageAssessmentError,
    admit_recovery_stage_bytes, assess_recovery_stage,
};

use super::{CATALOG_HEX, HEAD_HEX, SEGMENT_HEX, evidence, fixture, maximum_policy};

#[test]
fn every_fixed_stage_dispatches_to_its_only_semantic_classifier() -> Result<(), Box<dyn Error>> {
    let segment = fixture(SEGMENT_HEX)?;
    let catalog = fixture(CATALOG_HEX)?;
    let head = fixture(HEAD_HEX)?;
    let segment_evidence = evidence(RecoveryStage::Segment, &segment)?;
    let catalog_evidence = evidence(RecoveryStage::Catalog, &catalog)?;
    let head_evidence = evidence(RecoveryStage::NextHead, &head)?;

    let segment = assess(RecoveryStage::Segment, segment_evidence, &segment)?;
    let catalog = assess(RecoveryStage::Catalog, catalog_evidence, &catalog)?;
    let head = assess(RecoveryStage::NextHead, head_evidence, &head)?;

    assert_eq!(segment.evidence(), segment_evidence);
    assert_eq!(catalog.evidence(), catalog_evidence);
    assert_eq!(head.evidence(), head_evidence);
    assert!(matches!(
        segment,
        RecoveryStageAssessment::Segment {
            state: RecoverySegmentStage::Complete(_),
            ..
        }
    ));
    assert!(matches!(
        catalog,
        RecoveryStageAssessment::Catalog {
            state: RecoveryCatalogStage::Complete(_),
            ..
        }
    ));
    assert!(matches!(
        head,
        RecoveryStageAssessment::NextHead {
            state: RecoveryNextHeadStage::Complete(_),
            ..
        }
    ));
    Ok(())
}

#[test]
fn matching_evidence_does_not_sanitize_corrupt_content() -> Result<(), Box<dyn Error>> {
    let mut catalog = fixture(CATALOG_HEX)?;
    let byte = catalog.last_mut().ok_or("missing catalog checksum")?;
    *byte ^= 1;
    let observed = evidence(RecoveryStage::Catalog, &catalog)?;
    let admitted = admit_recovery_stage_bytes(RecoveryStage::Catalog, observed, &catalog)?;

    let error = assess_recovery_stage(&admitted, maximum_policy())
        .err()
        .ok_or("fingerprint-bound corrupt catalog was classified as lawful")?;

    assert!(matches!(
        error,
        RecoveryStageAssessmentError::Catalog {
            source: keep::RecoveryCatalogStageError::Complete {
                source: CatalogDecodeError::ChecksumMismatch { .. }
                    | CatalogDecodeError::DigestMismatch { .. },
            },
        }
    ));
    Ok(())
}

fn assess(
    stage: RecoveryStage,
    observed: keep::RecoveryStageEvidence,
    encoded: &[u8],
) -> Result<RecoveryStageAssessment<'_>, Box<dyn Error>> {
    let admitted = admit_recovery_stage_bytes(stage, observed, encoded)?;
    Ok(assess_recovery_stage(&admitted, maximum_policy())?)
}
