//! This boundary module owns one decoded and admitted retention manifest.

use super::{RetentionManifestDecodeError, manifest_decoder};
use crate::{RetentionManifest, RetentionManifestDigest};

/// Borrowed canonical bytes paired with their admitted semantic manifest.
///
/// Decoding verifies exact framing, the complete-record checksum, manifest and
/// entry-set digests, ordered entries, resource bounds, and generation-history
/// invariants. Entry allocation is bounded by a verified count. Decoding
/// performs no I/O.
#[must_use]
#[derive(Debug, Eq, PartialEq)]
pub struct AdmittedRetentionManifest<'encoded> {
    encoded: &'encoded [u8],
    manifest: RetentionManifest,
    digest: RetentionManifestDigest,
}

impl<'encoded> AdmittedRetentionManifest<'encoded> {
    /// Decodes and admits one exact canonical version-2 manifest record.
    ///
    /// # Errors
    ///
    /// Returns [`RetentionManifestDecodeError`] at the first violated framing,
    /// integrity, resource-bound, ordering, or semantic invariant.
    pub fn decode(encoded: &'encoded [u8]) -> Result<Self, RetentionManifestDecodeError> {
        manifest_decoder::decode(encoded)
    }

    /// Returns the complete verified canonical bytes.
    #[must_use]
    pub const fn encoded(&self) -> &'encoded [u8] {
        self.encoded
    }

    /// Returns the admitted semantic manifest.
    pub const fn manifest(&self) -> &RetentionManifest {
        &self.manifest
    }

    /// Returns the verified canonical manifest digest.
    pub const fn digest(&self) -> RetentionManifestDigest {
        self.digest
    }

    pub(super) const fn admitted(
        encoded: &'encoded [u8],
        manifest: RetentionManifest,
        digest: RetentionManifestDigest,
    ) -> Self {
        Self {
            encoded,
            manifest,
            digest,
        }
    }
}
