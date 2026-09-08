//! This module owns immutable-pool inventory identity.

/// Digest coordinate naming one canonical complete immutable-pool inventory.
///
/// Intent admission does not prove that a current inventory has this digest.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ImmutablePoolInventoryDigest([u8; 32]);

impl ImmutablePoolInventoryDigest {
    /// Returns the exact digest bytes.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub(super) const fn from_admitted(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}
