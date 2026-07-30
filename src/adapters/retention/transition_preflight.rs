//! This boundary module owns complete retention transition preflight.

use super::{
    AdmittedRetentionRoot, RetentionTransitionPreflightError, RetentionTransitionReadiness,
    VerifiedRetentionClosure, plan_retention_transition, verify_retention_closure,
};
use crate::retention::RetentionGenerationExpectation;
use crate::{CatalogSnapshot, RootGeneration};

/// Complete storage-independent proof required before retention publication.
#[must_use = "retention preflight must be consumed by publication or handled explicitly"]
#[derive(Debug)]
pub enum RetentionTransitionPreflight<'encoded> {
    /// The candidate is an exact successor whose verified closure must publish.
    Publish {
        /// Caller-supplied expected namespace generation.
        expected: RetentionGenerationExpectation,
        /// Namespace generation observed during transition planning.
        observed: Option<RootGeneration>,
        /// Fully admitted canonical candidate root.
        candidate: AdmittedRetentionRoot<'encoded>,
        /// Closure proof against the exact pinned catalog.
        closure: VerifiedRetentionClosure,
    },
    /// The exact candidate is current and its closure still verifies.
    AlreadyCommitted {
        /// Caller-supplied expected namespace generation.
        expected: RetentionGenerationExpectation,
        /// Namespace generation observed during transition planning.
        observed: Option<RootGeneration>,
        /// Fully admitted byte-identical current root.
        candidate: AdmittedRetentionRoot<'encoded>,
        /// Current closure proof against the exact pinned catalog.
        closure: VerifiedRetentionClosure,
    },
}

impl<'encoded> RetentionTransitionPreflight<'encoded> {
    /// Returns the caller-supplied expected namespace generation.
    pub const fn expected(&self) -> RetentionGenerationExpectation {
        match self {
            Self::Publish { expected, .. } | Self::AlreadyCommitted { expected, .. } => *expected,
        }
    }

    /// Returns the namespace generation observed during transition planning.
    pub const fn observed(&self) -> Option<RootGeneration> {
        match self {
            Self::Publish { observed, .. } | Self::AlreadyCommitted { observed, .. } => *observed,
        }
    }

    /// Borrows the fully admitted candidate root.
    pub const fn candidate(&self) -> &AdmittedRetentionRoot<'encoded> {
        match self {
            Self::Publish { candidate, .. } | Self::AlreadyCommitted { candidate, .. } => candidate,
        }
    }

    /// Returns the complete verified closure evidence.
    pub const fn closure(&self) -> VerifiedRetentionClosure {
        match self {
            Self::Publish { closure, .. } | Self::AlreadyCommitted { closure, .. } => *closure,
        }
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
    let observed = current.map(|root| root.root().generation());
    let readiness = plan_retention_transition(expected, current, candidate)
        .map_err(|source| RetentionTransitionPreflightError::Transition { source })?;
    let closure =
        verify_retention_closure(readiness.candidate().root(), catalog).map_err(|source| {
            RetentionTransitionPreflightError::Closure {
                source: Box::new(source),
            }
        })?;
    Ok(match readiness {
        RetentionTransitionReadiness::Publish { candidate } => {
            RetentionTransitionPreflight::Publish {
                expected,
                observed,
                candidate,
                closure,
            }
        }
        RetentionTransitionReadiness::AlreadyCommitted { candidate } => {
            RetentionTransitionPreflight::AlreadyCommitted {
                expected,
                observed,
                candidate,
                closure,
            }
        }
    })
}
