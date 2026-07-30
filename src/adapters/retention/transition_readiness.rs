//! This boundary module owns admitted retention transition readiness.

use super::AdmittedRetentionRoot;

/// Result of comparing one expected, observed, and candidate root.
#[must_use]
#[derive(Debug, Eq, PartialEq)]
pub enum RetentionTransitionReadiness<'encoded> {
    /// The candidate is the exact next root and still requires publication.
    Publish {
        /// Fully admitted candidate root.
        candidate: AdmittedRetentionRoot<'encoded>,
    },
    /// The exact candidate bytes are already the current published root.
    AlreadyCommitted {
        /// Fully admitted byte-identical replay candidate.
        candidate: AdmittedRetentionRoot<'encoded>,
    },
}

impl<'encoded> RetentionTransitionReadiness<'encoded> {
    /// Borrows the fully admitted candidate root.
    pub const fn candidate(&self) -> &AdmittedRetentionRoot<'encoded> {
        match self {
            Self::Publish { candidate } | Self::AlreadyCommitted { candidate } => candidate,
        }
    }

    /// Consumes the readiness proof and returns the admitted candidate root.
    pub fn into_candidate(self) -> AdmittedRetentionRoot<'encoded> {
        match self {
            Self::Publish { candidate } | Self::AlreadyCommitted { candidate } => candidate,
        }
    }
}
