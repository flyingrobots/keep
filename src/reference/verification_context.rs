//! This module owns the request and stage behind a reference verification refusal.

use crate::{CorruptionEvidence, MissingEvidence, VerificationDepth, VerificationSubject};

/// Reference evidence accompanying the precise shared verification refusal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReferenceVerificationEvidence {
    /// The absent layout, blob binding or chunk.
    Missing(MissingEvidence),
    /// The exact integrity contradiction.
    Corrupt(CorruptionEvidence),
    /// The requested depth or subject is unsupported by the reference view.
    Unsupported,
}

/// Admitted request context from an unsuccessful reference verification.
///
/// The original subject may differ from the shared refusal's subject: verifying
/// a blob can discover a missing chunk. Private construction binds those facts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReferenceVerificationContext {
    subject: VerificationSubject,
    stage: VerificationDepth,
    evidence: ReferenceVerificationEvidence,
}

impl ReferenceVerificationContext {
    /// The subject the caller requested.
    pub const fn subject(self) -> VerificationSubject {
        self.subject
    }
    /// The stage where verification stopped.
    pub const fn stage(self) -> VerificationDepth {
        self.stage
    }
    /// The concrete reference-view evidence.
    pub const fn evidence(self) -> ReferenceVerificationEvidence {
        self.evidence
    }

    pub(crate) const fn semantic_refusal(self) -> crate::VerificationRefusal {
        super::verification_outcome::semantic_refusal(self)
    }

    pub(super) const fn new(
        subject: VerificationSubject,
        stage: VerificationDepth,
        evidence: ReferenceVerificationEvidence,
    ) -> Self {
        Self {
            subject,
            stage,
            evidence,
        }
    }
}
