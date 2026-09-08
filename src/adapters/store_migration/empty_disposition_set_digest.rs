//! This module owns the registered empty recovery-disposition-set identity.

/// Registered identity of an empty version-2 recovery-disposition set.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct EmptyDispositionSetDigest([u8; 32]);

impl EmptyDispositionSetDigest {
    /// Returns the exact digest bytes.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub(super) const fn from_hash(hash: [u8; 32]) -> Self {
        Self(hash)
    }
}
