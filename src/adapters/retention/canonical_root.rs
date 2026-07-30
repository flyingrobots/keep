//! This boundary module owns materialized canonical retention root bytes.

use super::{RetentionRootEncodeError, root_encoder};
use crate::{RetentionRoot, RetentionRootDigest};

/// Owned canonical version-2 retention root record.
///
/// The complete record is materialized in memory after semantic bounds are
/// admitted and exact checked length calculation succeeds.
#[must_use]
#[derive(Debug, Eq, PartialEq)]
pub struct CanonicalRetentionRoot {
    encoded: Vec<u8>,
    digest: RetentionRootDigest,
}

impl CanonicalRetentionRoot {
    /// Encodes one validated semantic retention root.
    ///
    /// # Errors
    ///
    /// Returns [`RetentionRootEncodeError`] for checked length overflow,
    /// allocation refusal, or an internal construction-length mismatch.
    pub fn from_root(root: &RetentionRoot) -> Result<Self, RetentionRootEncodeError> {
        root_encoder::encode(root)
    }

    /// Returns the complete canonical root bytes.
    #[must_use]
    pub fn encoded(&self) -> &[u8] {
        &self.encoded
    }

    /// Returns the canonical root digest embedded in the record.
    pub const fn digest(&self) -> RetentionRootDigest {
        self.digest
    }

    pub(super) const fn admitted(encoded: Vec<u8>, digest: RetentionRootDigest) -> Self {
        Self { encoded, digest }
    }
}
