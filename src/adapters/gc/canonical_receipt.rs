//! This boundary module owns canonical owned GC retirement receipt bytes.

use super::{
    CanonicalGcRetirementIntent, GcRetirementReceipt, PoolStateDigest, receipt_encoder,
    receipt_format,
};

/// Owned canonical GC retirement receipt record.
///
/// Construction binds the completed intent and the caller's post-retirement
/// pool-state digest. It does not prove that any candidate was unlinked or
/// that any directory was synchronized.
#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalGcRetirementReceipt {
    encoded: [u8; receipt_format::ENCODED_LENGTH],
    receipt: GcRetirementReceipt,
}

impl CanonicalGcRetirementReceipt {
    /// Constructs the one receipt that completes `intent`.
    pub fn from_intent(
        intent: &CanonicalGcRetirementIntent,
        pool_state_digest: PoolStateDigest,
    ) -> Self {
        receipt_encoder::encode(GcRetirementReceipt::for_intent(
            intent.digest(),
            intent.candidate_set_digest(),
            intent.intent(),
            pool_state_digest,
        ))
    }

    /// Returns the exact canonical receipt bytes.
    #[must_use]
    pub const fn encoded(&self) -> &[u8] {
        &self.encoded
    }

    /// Returns the semantic receipt.
    pub const fn receipt(&self) -> &GcRetirementReceipt {
        &self.receipt
    }

    pub(super) const fn admitted(
        encoded: &[u8; receipt_format::ENCODED_LENGTH],
        receipt: GcRetirementReceipt,
    ) -> Self {
        Self {
            encoded: *encoded,
            receipt,
        }
    }
}
