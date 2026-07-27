//! Validated flat-layout identity.

use std::fmt;

use super::{LayoutIdMismatch, LayoutRecordLength};

/// A canonical identity for one exact flat-layout record.
///
/// Parsing proves only canonical coordinate shape. It does not prove that the
/// named record is present, structurally valid, admitted, or content-verified.
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct LayoutId {
    plan_length: LayoutRecordLength,
    digest: [u8; 32],
}

impl LayoutId {
    pub(crate) const fn from_validated_parts(
        plan_length: LayoutRecordLength,
        digest: [u8; 32],
    ) -> Self {
        Self {
            plan_length,
            digest,
        }
    }

    /// Returns the exact canonical record length committed by this identity.
    #[must_use]
    pub const fn plan_length(self) -> LayoutRecordLength {
        self.plan_length
    }

    /// Requires this observed coordinate to match an independently expected
    /// coordinate.
    ///
    /// Plan length is compared before digest so callers receive deterministic
    /// mismatch classification.
    ///
    /// # Errors
    ///
    /// Returns [`LayoutIdMismatch::PlanLength`] for different record lengths,
    /// otherwise [`LayoutIdMismatch::Digest`] for different digests.
    pub fn verify_expected(self, expected: Self) -> Result<(), LayoutIdMismatch> {
        if self.plan_length != expected.plan_length {
            return Err(LayoutIdMismatch::PlanLength {
                expected: expected.plan_length,
                observed: self.plan_length,
            });
        }
        if self.digest != expected.digest {
            return Err(LayoutIdMismatch::Digest {
                expected: expected.digest,
                observed: self.digest,
            });
        }
        Ok(())
    }

    pub(crate) const fn digest(&self) -> &[u8; 32] {
        &self.digest
    }
}

impl fmt::Debug for LayoutId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("LayoutId")
            .field("plan_length", &self.plan_length)
            .field("digest", &self.digest)
            .finish()
    }
}
