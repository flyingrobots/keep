//! This boundary module owns admitted retention transition readiness.

use super::{AdmittedRetentionRoot, RetentionTransitionDisposition};
use crate::{RetentionGenerationExpectation, RootGeneration};

/// Unforgeable result of comparing expected, observed, and candidate state.
#[must_use]
#[derive(Debug, Eq, PartialEq)]
pub struct RetentionTransitionReadiness<'encoded> {
    disposition: RetentionTransitionDisposition,
    expected: RetentionGenerationExpectation,
    observed: Option<RootGeneration>,
    candidate: AdmittedRetentionRoot<'encoded>,
}

impl<'encoded> RetentionTransitionReadiness<'encoded> {
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

    /// Consumes the readiness proof and returns the admitted candidate root.
    pub fn into_candidate(self) -> AdmittedRetentionRoot<'encoded> {
        self.candidate
    }

    pub(super) const fn publish(
        expected: RetentionGenerationExpectation,
        observed: Option<RootGeneration>,
        candidate: AdmittedRetentionRoot<'encoded>,
    ) -> Self {
        Self {
            disposition: RetentionTransitionDisposition::Publish,
            expected,
            observed,
            candidate,
        }
    }

    pub(super) const fn already_committed(
        expected: RetentionGenerationExpectation,
        observed: Option<RootGeneration>,
        candidate: AdmittedRetentionRoot<'encoded>,
    ) -> Self {
        Self {
            disposition: RetentionTransitionDisposition::AlreadyCommitted,
            expected,
            observed,
            candidate,
        }
    }

    pub(super) fn into_parts(
        self,
    ) -> (
        RetentionTransitionDisposition,
        RetentionGenerationExpectation,
        Option<RootGeneration>,
        AdmittedRetentionRoot<'encoded>,
    ) {
        (
            self.disposition,
            self.expected,
            self.observed,
            self.candidate,
        )
    }
}
