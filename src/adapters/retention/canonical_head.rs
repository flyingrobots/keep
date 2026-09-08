//! This boundary module owns materialized canonical retention-head bytes.

use super::head_encoder;
use crate::RetentionHead;

/// Owned canonical version-2 global retention-head record.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CanonicalRetentionHead {
    encoded: [u8; 144],
}

impl CanonicalRetentionHead {
    /// Encodes one validated semantic retention head.
    pub fn from_head(head: &RetentionHead) -> Self {
        head_encoder::encode(head)
    }

    /// Returns the complete exact canonical bytes.
    #[must_use]
    pub const fn encoded(&self) -> &[u8; 144] {
        &self.encoded
    }

    pub(super) const fn admitted(encoded: [u8; 144]) -> Self {
        Self { encoded }
    }
}
