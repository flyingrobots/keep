//! This boundary module owns admitted borrowed GC retirement intent bytes.

use super::{
    GcCandidateSetDigest, GcRetirementIntent, GcRetirementIntentDecodeError,
    GcRetirementIntentDigest, intent_decoder,
};

/// Borrowed canonical GC retirement intent record.
///
/// Admission proves framing, checksum, intent digest, candidate-set digest,
/// and every semantic coordinate. It does not prove that any candidate is
/// unreachable or that any retirement occurred.
#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdmittedGcRetirementIntent<'encoded> {
    encoded: &'encoded [u8],
    intent: GcRetirementIntent,
    candidate_set_digest: GcCandidateSetDigest,
    digest: GcRetirementIntentDigest,
}

impl<'encoded> AdmittedGcRetirementIntent<'encoded> {
    /// Decodes and admits one exact retirement intent.
    ///
    /// # Errors
    ///
    /// Returns [`GcRetirementIntentDecodeError`] for invalid framing,
    /// integrity, ordering, bounds, or semantic coordinates.
    pub fn decode(encoded: &'encoded [u8]) -> Result<Self, GcRetirementIntentDecodeError> {
        intent_decoder::decode(encoded)
    }

    /// Returns the exact borrowed canonical bytes.
    #[must_use]
    pub const fn encoded(&self) -> &'encoded [u8] {
        self.encoded
    }

    /// Returns the semantic intent.
    pub const fn intent(&self) -> &GcRetirementIntent {
        &self.intent
    }

    /// Returns the verified candidate-set digest.
    pub const fn candidate_set_digest(&self) -> GcCandidateSetDigest {
        self.candidate_set_digest
    }

    /// Returns the verified identity of the header and candidate bytes.
    pub const fn digest(&self) -> GcRetirementIntentDigest {
        self.digest
    }

    pub(super) const fn admitted(
        encoded: &'encoded [u8],
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
