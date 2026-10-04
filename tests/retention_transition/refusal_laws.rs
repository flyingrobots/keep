//! Exact stale, mismatch, and exhaustion transition refusals.

use keep::{
    AdmittedRetentionRoot, RetentionGenerationExpectation, RetentionTransitionError,
    RootGeneration, RootGenerationError, plan_retention_transition,
};

use super::{candidate, fixture_bytes};

#[test]
fn stale_expected_state_reports_both_coordinates() -> Result<(), Box<dyn std::error::Error>> {
    let bytes = fixture_bytes()?;
    let candidate = AdmittedRetentionRoot::decode(&bytes)?;
    let expected = RetentionGenerationExpectation::Current(RootGeneration::new(1)?);
    assert!(matches!(
        plan_retention_transition(expected, None, candidate),
        Err(RetentionTransitionError::StaleGeneration {
            expected: error_expected,
            observed: None,
        }) if error_expected == expected
    ));
    Ok(())
}

#[test]
fn successor_requires_the_same_namespace_generation_and_predecessor()
-> Result<(), Box<dyn std::error::Error>> {
    let bytes = fixture_bytes()?;
    let current = AdmittedRetentionRoot::decode(&bytes)?;
    let expected = RetentionGenerationExpectation::Current(current.root().generation());

    let wrong_namespace = candidate(
        &current,
        b"different",
        current.root().generation().successor()?,
        Some(current.digest()),
    )?;
    let candidate_root = AdmittedRetentionRoot::decode(wrong_namespace.encoded())?;
    assert!(matches!(
        plan_retention_transition(expected, Some(&current), candidate_root),
        Err(RetentionTransitionError::NamespaceMismatch { .. })
    ));

    let wrong_generation = candidate(
        &current,
        current.root().namespace().as_bytes(),
        current.root().generation().successor()?.successor()?,
        Some(current.digest()),
    )?;
    let candidate_root = AdmittedRetentionRoot::decode(wrong_generation.encoded())?;
    assert!(matches!(
        plan_retention_transition(expected, Some(&current), candidate_root),
        Err(RetentionTransitionError::CandidateGeneration {
            expected,
            observed,
        }) if expected.get() == 2 && observed.get() == 3
    ));

    let other_initial = candidate(&current, b"other", RootGeneration::new(1)?, None)?;
    let other = AdmittedRetentionRoot::decode(other_initial.encoded())?;
    let wrong_predecessor = candidate(
        &current,
        current.root().namespace().as_bytes(),
        current.root().generation().successor()?,
        Some(other.digest()),
    )?;
    let candidate_root = AdmittedRetentionRoot::decode(wrong_predecessor.encoded())?;
    assert!(matches!(
        plan_retention_transition(expected, Some(&current), candidate_root),
        Err(RetentionTransitionError::CandidatePredecessor { .. })
    ));
    Ok(())
}

#[test]
fn maximum_current_generation_has_no_transition() -> Result<(), Box<dyn std::error::Error>> {
    let bytes = fixture_bytes()?;
    let initial = AdmittedRetentionRoot::decode(&bytes)?;
    let maximum = candidate(
        &initial,
        initial.root().namespace().as_bytes(),
        RootGeneration::new(u64::MAX)?,
        Some(initial.digest()),
    )?;
    let current = AdmittedRetentionRoot::decode(maximum.encoded())?;
    let candidate = AdmittedRetentionRoot::decode(&bytes)?;
    assert!(matches!(
        plan_retention_transition(
            RetentionGenerationExpectation::Current(current.root().generation()),
            Some(&current),
            candidate,
        ),
        Err(RetentionTransitionError::GenerationExhausted {
            source: RootGenerationError::Exhausted { current: u64::MAX },
        })
    ));
    Ok(())
}
