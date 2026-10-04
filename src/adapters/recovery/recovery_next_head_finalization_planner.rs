//! This module owns complete next-head recovery planning.

use super::catalog_publication_expectation::ExpectedCurrentCatalog;
use super::catalog_transition;
use super::{
    CatalogPublicationExpectation, CatalogSnapshot, RecoveryNextHeadFinalizationPlanError,
    RecoveryNextHeadFinalizationRequest, RecoveryNextHeadFinalizationTarget, RecoveryNextHeadStage,
    RecoveryStageAssessment,
};

/// Plans exact, transition-checked finalization of one complete `head.next`.
///
/// The candidate snapshot must prove the complete transitive catalog view
/// named by the assessed head. The returned request owns only bounded evidence,
/// the expected current coordinate, and the validated candidate coordinate.
///
/// # Errors
///
/// Returns [`RecoveryNextHeadFinalizationPlanError`] unless the assessment is a
/// complete `head.next` whose snapshot is generation one over an uninitialized
/// root or the exact successor of the expected current snapshot.
pub fn plan_recovery_next_head_finalization(
    assessment: &RecoveryStageAssessment<'_>,
    candidate: &CatalogSnapshot<'_, '_, '_>,
    expectation: CatalogPublicationExpectation,
) -> Result<RecoveryNextHeadFinalizationRequest, RecoveryNextHeadFinalizationPlanError> {
    let head = match assessment {
        RecoveryStageAssessment::NextHead {
            state: RecoveryNextHeadStage::Complete(head),
            ..
        } => head,
        RecoveryStageAssessment::NextHead { .. } => {
            return Err(RecoveryNextHeadFinalizationPlanError::NotComplete);
        }
        RecoveryStageAssessment::Segment { .. } | RecoveryStageAssessment::Catalog { .. } => {
            return Err(RecoveryNextHeadFinalizationPlanError::NotNextHead {
                stage: assessment.evidence().stage(),
            });
        }
    };
    verify_snapshot(head, candidate)?;
    verify_transition(expectation, candidate)?;
    let target = RecoveryNextHeadFinalizationTarget::from_snapshot(candidate);
    Ok(RecoveryNextHeadFinalizationRequest::new(
        assessment.evidence(),
        expectation,
        target,
    ))
}

fn verify_snapshot(
    head: &super::ChecksummedPublicationHead<'_>,
    candidate: &CatalogSnapshot<'_, '_, '_>,
) -> Result<(), RecoveryNextHeadFinalizationPlanError> {
    let coordinate_matches = head.generation() == candidate.generation()
        && head.catalog_length() == candidate.catalog_length()
        && head.catalog_digest() == candidate.catalog_digest();
    if coordinate_matches {
        return Ok(());
    }
    Err(RecoveryNextHeadFinalizationPlanError::SnapshotCoordinate {
        expected_generation: head.generation(),
        expected_length: head.catalog_length(),
        expected_digest: head.catalog_digest(),
        observed_generation: candidate.generation(),
        observed_length: candidate.catalog_length(),
        observed_digest: candidate.catalog_digest(),
    })
}

fn verify_transition(
    expectation: CatalogPublicationExpectation,
    candidate: &CatalogSnapshot<'_, '_, '_>,
) -> Result<(), RecoveryNextHeadFinalizationPlanError> {
    match expectation.current() {
        ExpectedCurrentCatalog::Uninitialized => {
            if candidate.generation().get() == 1 {
                Ok(())
            } else {
                Err(RecoveryNextHeadFinalizationPlanError::InitialGeneration {
                    observed: candidate.generation(),
                })
            }
        }
        ExpectedCurrentCatalog::Published { generation, digest } => {
            catalog_transition::validate_coordinates(
                generation,
                digest,
                candidate.generation(),
                candidate.previous_catalog_digest(),
            )
            .map_err(|source| RecoveryNextHeadFinalizationPlanError::Transition { source })
        }
    }
}
