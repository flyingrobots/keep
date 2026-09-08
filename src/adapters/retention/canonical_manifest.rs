//! This boundary module owns materialized canonical retention manifest bytes.

use super::{RetentionManifestEncodeError, manifest_encoder};
use crate::{RetentionManifest, RetentionManifestDigest};

/// Owned canonical version-2 retention manifest record.
///
/// The complete record is materialized in memory after semantic bounds are
/// admitted and exact checked length calculation succeeds.
#[must_use]
#[derive(Debug, Eq, PartialEq)]
pub struct CanonicalRetentionManifest {
    encoded: Vec<u8>,
    digest: RetentionManifestDigest,
}

impl CanonicalRetentionManifest {
    /// Encodes one validated semantic retention manifest.
    ///
    /// # Errors
    ///
    /// Returns [`RetentionManifestEncodeError`] for checked length overflow,
    /// allocation refusal, or an internal construction-length mismatch.
    pub fn from_manifest(
        manifest: &RetentionManifest,
    ) -> Result<Self, RetentionManifestEncodeError> {
        manifest_encoder::encode(manifest)
    }

    /// Returns the complete canonical manifest bytes.
    #[must_use]
    pub fn encoded(&self) -> &[u8] {
        &self.encoded
    }

    /// Returns the canonical manifest digest embedded in the record.
    pub const fn digest(&self) -> RetentionManifestDigest {
        self.digest
    }

    pub(super) const fn admitted(encoded: Vec<u8>, digest: RetentionManifestDigest) -> Self {
        Self { encoded, digest }
    }
}
