//! This boundary module owns admitted borrowed GC retirement receipt bytes.

use super::{
    AdmittedGcRetirementIntent, GcRetirementReceipt, GcRetirementReceiptDecodeError,
    receipt_decoder,
};

/// Borrowed canonical GC retirement receipt record.
///
/// Admission proves framing, checksum, and exact binding to the supplied
/// admitted intent. It does not prove that the named retirement occurred;
/// GC recovery must establish that from the pool.
#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdmittedGcRetirementReceipt<'encoded> {
    encoded: &'encoded [u8],
    receipt: GcRetirementReceipt,
}

impl<'encoded> AdmittedGcRetirementReceipt<'encoded> {
    /// Decodes and admits one exact receipt bound to `intent`.
    ///
    /// # Errors
    ///
    /// Returns [`GcRetirementReceiptDecodeError`] for invalid framing,
    /// integrity, or any coordinate that disagrees with the intent.
    pub fn decode(
        encoded: &'encoded [u8],
        intent: &AdmittedGcRetirementIntent<'_>,
    ) -> Result<Self, GcRetirementReceiptDecodeError> {
        receipt_decoder::decode(encoded, intent)
    }

    /// Returns the exact borrowed canonical bytes.
    #[must_use]
    pub const fn encoded(&self) -> &'encoded [u8] {
        self.encoded
    }

    /// Returns the semantic receipt.
    pub const fn receipt(&self) -> &GcRetirementReceipt {
        &self.receipt
    }

    pub(super) const fn admitted(encoded: &'encoded [u8], receipt: GcRetirementReceipt) -> Self {
        Self { encoded, receipt }
    }
}
