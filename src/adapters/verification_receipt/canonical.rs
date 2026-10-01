//! This boundary module owns the canonical 384-byte verification receipt.

use super::decode_error::VerificationReceiptDecodeError;
use super::format::ENCODED_LENGTH;
use super::receipt::VerificationReceipt;
use super::{decoder, decoder_semantics, encoder};

/// One canonical, checksummed verification receipt.
///
/// `encode` is total: every `VerificationReceipt` has exactly one encoding.
/// `decode` admits framing, contract, checksum, registered codes, identity
/// slots, and every semantic law before returning the same bytes, so a
/// receipt written by one process is admitted by another exactly as it was
/// meant. Keep does not persist receipts; the application does.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CanonicalVerificationReceipt {
    encoded: [u8; ENCODED_LENGTH],
    receipt: VerificationReceipt,
}

impl CanonicalVerificationReceipt {
    /// The exact encoded length.
    pub const ENCODED_LENGTH: usize = ENCODED_LENGTH;

    /// Encodes one receipt canonically.
    pub fn encode(receipt: &VerificationReceipt) -> Self {
        Self {
            encoded: encoder::encode(receipt),
            receipt: *receipt,
        }
    }

    /// Decodes and admits one exact receipt.
    ///
    /// # Errors
    ///
    /// Returns [`VerificationReceiptDecodeError`] at the first framing,
    /// contract, checksum, code, identity, or semantic refusal.
    pub fn decode(encoded: &[u8]) -> Result<Self, VerificationReceiptDecodeError> {
        let fields = decoder::decode(encoded)?;
        let receipt = decoder_semantics::admit(&fields)?;
        let canonical = Self::encode(&receipt);
        if canonical.encoded.as_slice() == encoded {
            Ok(canonical)
        } else {
            Err(VerificationReceiptDecodeError::Semantic {
                law: "the bytes are not the canonical encoding of their receipt",
            })
        }
    }

    /// The exact canonical bytes.
    #[must_use]
    pub const fn encoded(&self) -> &[u8; ENCODED_LENGTH] {
        &self.encoded
    }

    /// The semantic receipt.
    pub const fn receipt(&self) -> &VerificationReceipt {
        &self.receipt
    }
}
