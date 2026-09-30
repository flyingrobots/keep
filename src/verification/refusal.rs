//! This module owns evidenced verification refusals.

use super::{CorruptionEvidence, MissingEvidence, VerificationDepth, VerificationSubject};

/// An evidenced refusal: the view is complete enough to say exactly why the
/// requested depth cannot be established.
///
/// `stage` names the depth Keep was establishing when it stopped. Missing,
/// corrupt, and ambiguous evidence are distinct, and none of them is ever
/// repaired, substituted, or quarantined by verification.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VerificationRefusal {
    /// Required evidence is absent from a complete view.
    Missing {
        /// Subject being verified.
        subject: VerificationSubject,
        /// Depth being established when the absence was found.
        stage: VerificationDepth,
        /// What was absent.
        evidence: MissingEvidence,
    },
    /// Present evidence contradicts the identity it must reproduce.
    Corrupt {
        /// Subject being verified.
        subject: VerificationSubject,
        /// Depth being established when the contradiction was found.
        stage: VerificationDepth,
        /// The exact contradiction.
        evidence: CorruptionEvidence,
    },
    /// Two pieces of admitted evidence conflict, so neither a positive nor a
    /// negative conclusion follows. No reference-store path produces this;
    /// durable views reserve it for conflicting catalog or retention state.
    Ambiguous {
        /// Subject being verified.
        subject: VerificationSubject,
        /// Depth being established when the conflict was found.
        stage: VerificationDepth,
    },
    /// The view cannot establish the requested depth at all, and refuses
    /// rather than reporting a shallower one.
    Unsupported {
        /// Subject requested.
        subject: VerificationSubject,
        /// Depth requested.
        requested: VerificationDepth,
        /// Shallowest depth this view establishes.
        supported_minimum: VerificationDepth,
        /// Deepest depth this view establishes.
        supported_maximum: VerificationDepth,
    },
}
