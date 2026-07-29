//! Next-head finalization planning laws.

use std::error::Error;

use keep::{
    CatalogPublicationExpectation, CatalogTransitionError, RecoveryNextHeadFinalizationPlanError,
    RecoveryNextHeadFinalizationTarget, RecoveryStage, plan_recovery_next_head_finalization,
};

use super::{
    CATALOG_ONE_HEX, CATALOG_TWO_HEX, HEAD_ONE_HEX, HEAD_TWO_HEX, SEGMENT_HEX, assessment, fixture,
    snapshot,
};

#[test]
fn generation_one_candidate_extends_an_uninitialized_root() -> Result<(), Box<dyn Error>> {
    let segment = fixture(SEGMENT_HEX)?;
    let catalog = fixture(CATALOG_ONE_HEX)?;
    let head = fixture(HEAD_ONE_HEX)?;
    let assessed = assessment(RecoveryStage::NextHead, &head)?;
    let candidate = snapshot(&head, &catalog, &segment)?;

    let request = plan_recovery_next_head_finalization(
        &assessed,
        &candidate,
        CatalogPublicationExpectation::uninitialized(),
    )?;

    assert_eq!(request.evidence(), assessed.evidence());
    assert_eq!(
        request.target(),
        RecoveryNextHeadFinalizationTarget::from_snapshot(&candidate)
    );
    Ok(())
}

#[test]
fn generation_two_candidate_extends_the_exact_current_snapshot() -> Result<(), Box<dyn Error>> {
    let segment = fixture(SEGMENT_HEX)?;
    let catalog_one = fixture(CATALOG_ONE_HEX)?;
    let head_one = fixture(HEAD_ONE_HEX)?;
    let catalog_two = fixture(CATALOG_TWO_HEX)?;
    let head_two = fixture(HEAD_TWO_HEX)?;
    let current = snapshot(&head_one, &catalog_one, &segment)?;
    let candidate = snapshot(&head_two, &catalog_two, &segment)?;
    let assessed = assessment(RecoveryStage::NextHead, &head_two)?;
    let expectation = CatalogPublicationExpectation::successor_of(&current);

    let request = plan_recovery_next_head_finalization(&assessed, &candidate, expectation)?;

    assert_eq!(request.expectation(), expectation);
    assert_eq!(request.target().generation(), candidate.generation());
    assert_eq!(
        candidate.previous_catalog_digest(),
        expectation.current_catalog_digest()
    );
    Ok(())
}

#[test]
fn candidate_snapshot_must_match_the_assessed_head() -> Result<(), Box<dyn Error>> {
    let segment = fixture(SEGMENT_HEX)?;
    let head_one = fixture(HEAD_ONE_HEX)?;
    let catalog_two = fixture(CATALOG_TWO_HEX)?;
    let head_two = fixture(HEAD_TWO_HEX)?;
    let assessed = assessment(RecoveryStage::NextHead, &head_one)?;
    let wrong = snapshot(&head_two, &catalog_two, &segment)?;

    let error = plan_recovery_next_head_finalization(
        &assessed,
        &wrong,
        CatalogPublicationExpectation::uninitialized(),
    )
    .err()
    .ok_or("mismatched candidate snapshot was accepted")?;

    assert!(matches!(
        error,
        RecoveryNextHeadFinalizationPlanError::SnapshotCoordinate {
            expected_generation,
            observed_generation,
            ..
        } if expected_generation.get() == 1 && observed_generation.get() == 2
    ));
    Ok(())
}

#[test]
fn noninitial_candidate_cannot_extend_an_uninitialized_root() -> Result<(), Box<dyn Error>> {
    let segment = fixture(SEGMENT_HEX)?;
    let catalog = fixture(CATALOG_TWO_HEX)?;
    let head = fixture(HEAD_TWO_HEX)?;
    let assessed = assessment(RecoveryStage::NextHead, &head)?;
    let candidate = snapshot(&head, &catalog, &segment)?;

    let error = plan_recovery_next_head_finalization(
        &assessed,
        &candidate,
        CatalogPublicationExpectation::uninitialized(),
    )
    .err()
    .ok_or("generation-two candidate extended an uninitialized root")?;

    assert!(matches!(
        error,
        RecoveryNextHeadFinalizationPlanError::InitialGeneration { observed }
            if observed.get() == 2
    ));
    Ok(())
}

#[test]
fn candidate_must_be_the_exact_expected_successor() -> Result<(), Box<dyn Error>> {
    let segment = fixture(SEGMENT_HEX)?;
    let catalog = fixture(CATALOG_ONE_HEX)?;
    let head = fixture(HEAD_ONE_HEX)?;
    let current = snapshot(&head, &catalog, &segment)?;
    let assessed = assessment(RecoveryStage::NextHead, &head)?;
    let expectation = CatalogPublicationExpectation::successor_of(&current);

    let error = plan_recovery_next_head_finalization(&assessed, &current, expectation)
        .err()
        .ok_or("current generation was accepted as its own successor")?;

    assert!(matches!(
        error,
        RecoveryNextHeadFinalizationPlanError::Transition {
            source: CatalogTransitionError::Generation { expected, observed },
        } if expected.get() == 2 && observed.get() == 1
    ));
    Ok(())
}

#[test]
fn only_a_complete_next_head_can_enter_finalization() -> Result<(), Box<dyn Error>> {
    let segment = fixture(SEGMENT_HEX)?;
    let catalog = fixture(CATALOG_ONE_HEX)?;
    let head = fixture(HEAD_ONE_HEX)?;
    let candidate = snapshot(&head, &catalog, &segment)?;
    let truncated = assessment(RecoveryStage::NextHead, &[0_u8])?;
    let wrong_stage = assessment(RecoveryStage::Catalog, &[0_u8])?;

    let truncated_error = plan_recovery_next_head_finalization(
        &truncated,
        &candidate,
        CatalogPublicationExpectation::uninitialized(),
    )
    .err()
    .ok_or("truncated next head entered finalization")?;
    let wrong_error = plan_recovery_next_head_finalization(
        &wrong_stage,
        &candidate,
        CatalogPublicationExpectation::uninitialized(),
    )
    .err()
    .ok_or("non-head stage entered finalization")?;

    assert_eq!(
        truncated_error,
        RecoveryNextHeadFinalizationPlanError::NotComplete
    );
    assert_eq!(
        wrong_error,
        RecoveryNextHeadFinalizationPlanError::NotNextHead {
            stage: RecoveryStage::Catalog,
        }
    );
    Ok(())
}
