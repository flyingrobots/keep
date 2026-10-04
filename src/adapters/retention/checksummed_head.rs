//! This boundary module owns a framing- and checksum-verified retention head.

use super::{RetentionHeadDecodeError, head_decoder};
use crate::RetentionHead;

/// Borrowed canonical retention-head bytes with admitted semantic coordinates.
///
/// This state does not prove that the named manifest exists or that its entries
/// name admitted namespace roots. A reader must bind those artifacts before
/// treating this value as a complete retention snapshot.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ChecksummedRetentionHead<'encoded> {
    encoded: &'encoded [u8],
    head: RetentionHead,
}

impl<'encoded> ChecksummedRetentionHead<'encoded> {
    /// Decodes exact version-2 framing and verifies the head checksum.
    ///
    /// This operation performs no allocation or I/O.
    ///
    /// # Errors
    ///
    /// Returns [`RetentionHeadDecodeError`] for wrong framing, unsupported or
    /// noncanonical fields, checksum disagreement, or invalid coordinates.
    pub fn decode(encoded: &'encoded [u8]) -> Result<Self, RetentionHeadDecodeError> {
        head_decoder::decode(encoded)
    }

    /// Returns the exact borrowed canonical bytes.
    #[must_use]
    pub const fn encoded(&self) -> &'encoded [u8] {
        self.encoded
    }

    /// Returns the admitted semantic head.
    pub const fn head(&self) -> &RetentionHead {
        &self.head
    }

    pub(super) const fn admitted(encoded: &'encoded [u8], head: RetentionHead) -> Self {
        Self { encoded, head }
    }
}
