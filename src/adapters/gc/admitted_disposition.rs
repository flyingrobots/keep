//! This boundary module owns admitted borrowed recovery-disposition bytes.

use super::{RecoveryDispositionDecodeError, RecoveryDispositionReceipt, disposition_decoder};

/// Borrowed canonical recovery-disposition receipt record.
///
/// Admission proves framing, checksum, and that every enumeration carries a
/// registered code. It does not prove that the named artifact exists, that
/// its coordinates still hold, or that the decision was executed; the
/// disposition protocol and GC planning revalidate those.
#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdmittedRecoveryDispositionReceipt<'encoded> {
    encoded: &'encoded [u8],
    receipt: RecoveryDispositionReceipt,
}

impl<'encoded> AdmittedRecoveryDispositionReceipt<'encoded> {
    /// Decodes and admits one exact receipt.
    ///
    /// # Errors
    ///
    /// Returns [`RecoveryDispositionDecodeError`] for invalid framing,
    /// integrity, an unregistered enumeration code, or a zero generation.
    pub fn decode(encoded: &'encoded [u8]) -> Result<Self, RecoveryDispositionDecodeError> {
        disposition_decoder::decode(encoded)
    }

    /// Returns the exact borrowed canonical bytes.
    #[must_use]
    pub const fn encoded(&self) -> &'encoded [u8] {
        self.encoded
    }

    /// Returns the semantic receipt.
    pub const fn receipt(&self) -> &RecoveryDispositionReceipt {
        &self.receipt
    }

    pub(super) const fn admitted(
        encoded: &'encoded [u8],
        receipt: &RecoveryDispositionReceipt,
    ) -> Self {
        Self {
            encoded,
            receipt: *receipt,
        }
    }
}
