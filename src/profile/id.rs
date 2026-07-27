//! Canonical storage-profile identity.

use std::fmt;

/// A structurally canonical deterministic storage-profile identity.
///
/// Parsing proves only canonical version-1 coordinate shape. Use
/// [`RegisteredStorageProfile::admit`](super::RegisteredStorageProfile::admit)
/// to establish that this Keep version implements the named profile.
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StorageProfileId {
    digest: [u8; 32],
}

impl StorageProfileId {
    pub(crate) const fn from_validated_digest(digest: [u8; 32]) -> Self {
        Self { digest }
    }

    pub(crate) const fn digest(&self) -> &[u8; 32] {
        &self.digest
    }
}

impl fmt::Debug for StorageProfileId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("StorageProfileId")
            .field("digest", &self.digest)
            .finish()
    }
}
