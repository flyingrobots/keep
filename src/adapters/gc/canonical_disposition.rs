//! This boundary module owns canonical owned recovery-disposition bytes.

use super::{RecoveryDispositionReceipt, disposition_encoder, disposition_format};

/// Owned canonical recovery-disposition receipt record.
///
/// Construction encodes the decision exactly; it does not prove the
/// decision was executed or that the artifact still exists.
#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalRecoveryDispositionReceipt {
    encoded: [u8; disposition_format::ENCODED_LENGTH],
    receipt: RecoveryDispositionReceipt,
}

impl CanonicalRecoveryDispositionReceipt {
    /// Encodes `receipt` canonically.
    pub fn from_receipt(receipt: &RecoveryDispositionReceipt) -> Self {
        disposition_encoder::encode(receipt)
    }

    /// Digests exact artifact bytes under the registered artifact domain, the
    /// value the receipt's content digest must carry.
    pub fn artifact_content_digest(bytes: &[u8]) -> super::ArtifactContentDigest {
        super::ArtifactContentDigest::new(disposition_format::artifact_content_digest(bytes))
    }

    /// Returns the exact canonical receipt bytes.
    #[must_use]
    pub const fn encoded(&self) -> &[u8] {
        &self.encoded
    }

    /// Returns the semantic receipt.
    pub const fn receipt(&self) -> &RecoveryDispositionReceipt {
        &self.receipt
    }

    pub(super) const fn admitted(
        encoded: &[u8; disposition_format::ENCODED_LENGTH],
        receipt: &RecoveryDispositionReceipt,
    ) -> Self {
        Self {
            encoded: *encoded,
            receipt: *receipt,
        }
    }
}
