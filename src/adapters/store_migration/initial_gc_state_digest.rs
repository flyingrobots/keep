//! This module owns the registered initial garbage-collection-state identity.

/// Registered identity of empty version-2 garbage-collection state.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct InitialGcStateDigest([u8; 32]);

impl InitialGcStateDigest {
    /// Returns the exact digest bytes.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub(super) const fn from_hash(hash: [u8; 32]) -> Self {
        Self(hash)
    }
}
