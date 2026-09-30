//! This boundary module owns canonical owned GC retirement intent bytes.

use super::{
    GcCandidateSetDigest, GcRetirementIntent, GcRetirementIntentDigest,
    GcRetirementIntentEncodeError, intent_encoder,
};

/// Owned canonical GC retirement intent record.
///
/// Construction proves only that the bytes are the one canonical encoding of
/// the semantic intent. It does not prove that the intent was written,
/// synchronized, or acted on.
#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalGcRetirementIntent {
    encoded: Vec<u8>,
    intent: GcRetirementIntent,
    candidate_set_digest: GcCandidateSetDigest,
    digest: GcRetirementIntentDigest,
}

impl CanonicalGcRetirementIntent {
    /// Encodes one semantic intent into its canonical bytes.
    ///
    /// # Errors
    ///
    /// Returns [`GcRetirementIntentEncodeError`] when the bounded allocation
    /// is refused.
    pub fn from_intent(intent: &GcRetirementIntent) -> Result<Self, GcRetirementIntentEncodeError> {
        intent_encoder::encode(intent)
    }

    /// Returns the exact canonical bytes.
    #[must_use]
    pub fn encoded(&self) -> &[u8] {
        &self.encoded
    }

    /// Returns the semantic intent.
    pub const fn intent(&self) -> &GcRetirementIntent {
        &self.intent
    }

    /// Returns the verified candidate-set digest.
    pub const fn candidate_set_digest(&self) -> GcCandidateSetDigest {
        self.candidate_set_digest
    }

    /// Returns the identity of the header and candidate bytes.
    pub const fn digest(&self) -> GcRetirementIntentDigest {
        self.digest
    }

    pub(super) const fn admitted(
        encoded: Vec<u8>,
        intent: GcRetirementIntent,
        candidate_set_digest: GcCandidateSetDigest,
        digest: GcRetirementIntentDigest,
    ) -> Self {
        Self {
            encoded,
            intent,
            candidate_set_digest,
            digest,
        }
    }
}
