//! This module owns the registered initial retention-state identity.

/// Registered identity of empty version-2 retention state.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct InitialRetentionStateDigest([u8; 32]);

impl InitialRetentionStateDigest {
    /// Returns the exact digest bytes.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub(super) const fn from_hash(hash: [u8; 32]) -> Self {
        Self(hash)
    }
}
