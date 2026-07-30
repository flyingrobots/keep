//! This boundary module owns complete retention transition preflight.

use super::{
    AdmittedRetentionRoot, RetentionTransitionDisposition, RetentionTransitionPreflightError,
    VerifiedRetentionClosure, plan_retention_transition, verify_retention_closure,
};
use crate::{CatalogSnapshot, RetentionGenerationExpectation, RootGeneration};

/// Unforgeable storage-independent proof required before publication.
#[must_use = "retention preflight must be consumed by publication or handled explicitly"]
#[derive(Debug)]
pub struct RetentionTransitionPreflight<'encoded> {
    disposition: RetentionTransitionDisposition,
    expected: RetentionGenerationExpectation,
    observed: Option<RootGeneration>,
    candidate: AdmittedRetentionRoot<'encoded>,
    closure: VerifiedRetentionClosure,
}

impl<'encoded> RetentionTransitionPreflight<'encoded> {
    /// Returns whether the candidate requires publication or is current.
    pub const fn disposition(&self) -> RetentionTransitionDisposition {
        self.disposition
    }

    /// Returns the caller-supplied expected namespace generation.
    pub const fn expected(&self) -> RetentionGenerationExpectation {
        self.expected
    }

    /// Returns the namespace generation observed during transition planning.
    pub const fn observed(&self) -> Option<RootGeneration> {
        self.observed
    }

    /// Borrows the fully admitted candidate root.
    pub const fn candidate(&self) -> &AdmittedRetentionRoot<'encoded> {
        &self.candidate
    }

    /// Returns the complete verified closure evidence.
    pub const fn closure(&self) -> VerifiedRetentionClosure {
        self.closure
    }

    pub(super) fn into_parts(
        self,
    ) -> (
        RetentionTransitionDisposition,
        RetentionGenerationExpectation,
        Option<RootGeneration>,
        AdmittedRetentionRoot<'encoded>,
        VerifiedRetentionClosure,
    ) {
        (
            self.disposition,
            self.expected,
            self.observed,
            self.candidate,
            self.closure,
        )
    }
}

/// Proves generation and closure invariants before retention storage mutation.
///
/// Generation planning completes before closure traversal. Exact replay still
/// requires the current closure to verify against the pinned catalog. The
/// function performs no I/O and inherits closure verification's root-bounded
/// record index and per-layout entry allocation.
///
/// # Errors
///
/// Returns [`RetentionTransitionPreflightError::Transition`] for generation or
/// successor refusal, then [`RetentionTransitionPreflightError::Closure`] for
/// the first deterministic closure refusal.
pub fn preflight_retention_transition<'encoded>(
    expected: RetentionGenerationExpectation,
    current: Option<&AdmittedRetentionRoot<'_>>,
    candidate: AdmittedRetentionRoot<'encoded>,
    catalog: &CatalogSnapshot<'_, '_, '_>,
) -> Result<RetentionTransitionPreflight<'encoded>, RetentionTransitionPreflightError> {
    let readiness = plan_retention_transition(expected, current, candidate)
        .map_err(|source| RetentionTransitionPreflightError::Transition { source })?;
    let closure =
        verify_retention_closure(readiness.candidate().root(), catalog).map_err(|source| {
            RetentionTransitionPreflightError::Closure {
                source: Box::new(source),
            }
        })?;
    let (disposition, expected, observed, candidate) = readiness.into_parts();
    Ok(RetentionTransitionPreflight {
        disposition,
        expected,
        observed,
        candidate,
        closure,
    })
}
