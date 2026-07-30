//! This boundary module owns storage-independent retention transition planning.

use super::{AdmittedRetentionRoot, RetentionTransitionError, RetentionTransitionReadiness};
use crate::RootGeneration;
use crate::retention::RetentionGenerationExpectation;

/// Compares one expected, observed, and fully admitted candidate root.
///
/// The returned readiness performs no I/O and proves no closure availability
/// or durability. Exact byte-identical replay is admitted only while the
/// candidate remains the current root and the expectation names its prior
/// state.
///
/// # Errors
///
/// Returns [`RetentionTransitionError`] for stale state, namespace mismatch,
/// generation exhaustion, or a non-successor candidate.
pub fn plan_retention_transition<'encoded>(
    expected: RetentionGenerationExpectation,
    current: Option<&AdmittedRetentionRoot<'_>>,
    candidate: AdmittedRetentionRoot<'encoded>,
) -> Result<RetentionTransitionReadiness<'encoded>, RetentionTransitionError> {
    if is_exact_replay(expected, current, &candidate)? {
        return Ok(RetentionTransitionReadiness::AlreadyCommitted { candidate });
    }
    require_expected_state(expected, current)?;
    match current {
        Some(current) => validate_successor(current, &candidate)?,
        None => validate_initial(&candidate)?,
    }
    Ok(RetentionTransitionReadiness::Publish { candidate })
}

fn is_exact_replay(
    expected: RetentionGenerationExpectation,
    current: Option<&AdmittedRetentionRoot<'_>>,
    candidate: &AdmittedRetentionRoot<'_>,
) -> Result<bool, RetentionTransitionError> {
    let Some(current) = current else {
        return Ok(false);
    };
    if current.encoded() != candidate.encoded() {
        return Ok(false);
    }
    let candidate_generation = candidate.root().generation();
    match expected {
        RetentionGenerationExpectation::Absent => {
            Ok(candidate_generation == RootGeneration::INITIAL)
        }
        RetentionGenerationExpectation::Current(generation) => {
            let successor = generation
                .successor()
                .map_err(|source| RetentionTransitionError::GenerationExhausted { source })?;
            Ok(candidate_generation == successor)
        }
    }
}

fn require_expected_state(
    expected: RetentionGenerationExpectation,
    current: Option<&AdmittedRetentionRoot<'_>>,
) -> Result<(), RetentionTransitionError> {
    let observed = current.map(|root| root.root().generation());
    let matches = match expected {
        RetentionGenerationExpectation::Absent => observed.is_none(),
        RetentionGenerationExpectation::Current(generation) => observed == Some(generation),
    };
    if matches {
        Ok(())
    } else {
        Err(RetentionTransitionError::StaleGeneration { expected, observed })
    }
}

fn validate_initial(candidate: &AdmittedRetentionRoot<'_>) -> Result<(), RetentionTransitionError> {
    let observed = candidate.root().generation();
    if observed == RootGeneration::INITIAL {
        Ok(())
    } else {
        Err(RetentionTransitionError::CandidateGeneration {
            expected: RootGeneration::INITIAL,
            observed,
        })
    }
}

fn validate_successor(
    current: &AdmittedRetentionRoot<'_>,
    candidate: &AdmittedRetentionRoot<'_>,
) -> Result<(), RetentionTransitionError> {
    let expected_namespace = current.root().namespace().digest();
    let observed_namespace = candidate.root().namespace().digest();
    if observed_namespace != expected_namespace {
        return Err(RetentionTransitionError::NamespaceMismatch {
            expected: expected_namespace,
            observed: observed_namespace,
        });
    }
    let expected_generation = current
        .root()
        .generation()
        .successor()
        .map_err(|source| RetentionTransitionError::GenerationExhausted { source })?;
    let observed_generation = candidate.root().generation();
    if observed_generation != expected_generation {
        return Err(RetentionTransitionError::CandidateGeneration {
            expected: expected_generation,
            observed: observed_generation,
        });
    }
    let expected_predecessor = current.digest();
    let observed_predecessor = candidate.root().predecessor();
    if observed_predecessor != Some(expected_predecessor) {
        return Err(RetentionTransitionError::CandidatePredecessor {
            expected: expected_predecessor,
            observed: observed_predecessor,
        });
    }
    Ok(())
}
