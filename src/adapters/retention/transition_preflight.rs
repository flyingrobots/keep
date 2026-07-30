//! This boundary module owns complete retention transition preflight.

use super::{
    AdmittedRetentionRoot, RetentionTransitionPreflightError, RetentionTransitionReadiness,
    VerifiedRetentionClosure, plan_retention_transition, verify_retention_closure,
};
use crate::CatalogSnapshot;
use crate::retention::RetentionGenerationExpectation;

/// Complete storage-independent proof required before retention publication.
#[must_use = "retention preflight must be consumed by publication or handled explicitly"]
#[derive(Debug)]
pub enum RetentionTransitionPreflight<'encoded> {
    /// The candidate is an exact successor whose verified closure must publish.
    Publish {
        /// Fully admitted canonical candidate root.
        candidate: AdmittedRetentionRoot<'encoded>,
        /// Closure proof against the exact pinned catalog.
        closure: VerifiedRetentionClosure,
    },
    /// The exact candidate is current and its closure still verifies.
    AlreadyCommitted {
        /// Fully admitted byte-identical current root.
        candidate: AdmittedRetentionRoot<'encoded>,
        /// Current closure proof against the exact pinned catalog.
        closure: VerifiedRetentionClosure,
    },
}

impl<'encoded> RetentionTransitionPreflight<'encoded> {
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
            RetentionTransitionPreflight::Publish { candidate, closure }
        }
        RetentionTransitionReadiness::AlreadyCommitted { candidate } => {
            RetentionTransitionPreflight::AlreadyCommitted { candidate, closure }
        }
    })
}
