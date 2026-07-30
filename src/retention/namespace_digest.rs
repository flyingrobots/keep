//! This module owns the canonical retention namespace digest coordinate.

/// Canonical BLAKE3-256 identity of one exact retention namespace.
///
/// This coordinate selects a physical namespace directory. Authority still
/// requires the matching root record to contain the exact namespace bytes.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RetentionNamespaceDigest([u8; 32]);

impl RetentionNamespaceDigest {
    pub(super) const fn from_hash(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Returns the exact 32 digest bytes.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}
