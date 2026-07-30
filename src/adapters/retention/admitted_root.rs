//! This boundary module owns one decoded and admitted retention root.

use super::{RetentionRootDecodeError, root_decoder};
use crate::{RetentionRoot, RetentionRootDigest};

/// Borrowed canonical bytes paired with their admitted semantic root.
///
/// Decoding verifies exact framing, the complete-record checksum, the root and
/// anchor-set digests, every nested identity, canonical anchor order, and all
/// semantic invariants. Anchor and namespace allocation is bounded by fields
/// admitted from the record. Decoding performs no I/O.
#[must_use]
#[derive(Debug, Eq, PartialEq)]
pub struct AdmittedRetentionRoot<'encoded> {
    encoded: &'encoded [u8],
    root: RetentionRoot,
    digest: RetentionRootDigest,
}

impl<'encoded> AdmittedRetentionRoot<'encoded> {
    /// Decodes and admits one exact canonical version-2 root record.
    ///
    /// # Errors
    ///
    /// Returns [`RetentionRootDecodeError`] at the first violated framing,
    /// integrity, nested-codec, resource-bound, or semantic invariant.
    pub fn decode(encoded: &'encoded [u8]) -> Result<Self, RetentionRootDecodeError> {
        root_decoder::decode(encoded)
    }

    /// Returns the complete verified canonical bytes.
    #[must_use]
    pub const fn encoded(&self) -> &'encoded [u8] {
        self.encoded
    }

    /// Returns the admitted semantic root.
    pub const fn root(&self) -> &RetentionRoot {
        &self.root
    }

    /// Returns the verified canonical root digest.
    pub const fn digest(&self) -> RetentionRootDigest {
        self.digest
    }

    pub(super) const fn admitted(
        encoded: &'encoded [u8],
        root: RetentionRoot,
        digest: RetentionRootDigest,
    ) -> Self {
        Self {
            encoded,
            root,
            digest,
        }
    }
}
